use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct FileEntry {
    pub name: String,
    pub size: u64,
    pub is_dir: bool,
    pub mtime: u64, // Unix timestamp
}

#[cfg(feature = "ssr")]
pub mod srv {
    use super::*;
    use crate::auth::srv_err;
    use crate::hosts::{Host, establish_ssh_session};
    use leptos::prelude::*;
    use r2d2::Pool;
    use duckdb::DuckdbConnectionManager;
    use axum::{
        body::Body,
        extract::{Multipart, Query},
        http::{StatusCode, header},
        response::{Response, IntoResponse},
        Extension,
    };
    use std::io::Write;

    #[derive(Deserialize)]
    pub struct AccessParams {
        pub volume_id: String,
        pub path: String,
    }

    /// Finds a host where the volume is "Up-to-date" and its actual device path.
    pub async fn find_active_host_for_volume(
        pool: Pool<DuckdbConnectionManager>,
        volume_name: &str
    ) -> Result<(Host, String), ServerFnError> {
        let hosts = crate::hosts::list_hosts_internal(pool.clone()).await?;
        
        let controller = crate::hosts::get_controller_host_internal(pool.clone()).await?
            .ok_or_else(|| srv_err("No Linstor controller found"))?;
        
        let client = crate::storage::linstor::LinstorClient::new(&controller.address);
        
        let (node_name, device_path) = client.get_resource_placement(volume_name).await
            .map_err(|e| srv_err(&format!("Linstor placement error: {}", e)))?;
        
        // Find the host that matches Linstor's node name
        let target_host = hosts.into_iter()
            .find(|h| h.name == node_name || h.address == node_name)
            .ok_or_else(|| srv_err(&format!("Target node '{}' from Linstor not found in CloudLab hosts", node_name)))?;

        if target_host.status != "online" {
            return Err(srv_err(&format!("Target node '{}' is currently offline", node_name)));
        }

        Ok((target_host, device_path))
    }

    pub async fn list_files(
        host: &Host,
        root_path: &str
    ) -> Result<Vec<FileEntry>, String> {
        let sess = establish_ssh_session(host).map_err(|e| e.to_string())?;
        let sftp = sess.sftp().map_err(|e| e.to_string())?;
        
        let entries = sftp.readdir(std::path::Path::new(root_path)).map_err(|e| e.to_string())?;
        
        let mut files = Vec::new();
        for (path, stat) in entries {
            let name = path.file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("unknown")
                .to_string();
            
            if name == "." || name == ".." {
                continue;
            }

            files.push(FileEntry {
                name,
                size: stat.size.unwrap_or(0),
                is_dir: stat.is_dir(),
                mtime: stat.mtime.unwrap_or(0),
            });
        }

        // Sort: directories first, then alphabetically
        files.sort_by(|a, b| {
            if a.is_dir != b.is_dir {
                b.is_dir.cmp(&a.is_dir)
            } else {
                a.name.to_lowercase().cmp(&b.name.to_lowercase())
            }
        });

        Ok(files)
    }

    pub async fn create_folder(host: &Host, path: &str) -> Result<(), String> {
        let sess = establish_ssh_session(host).map_err(|e| e.to_string())?;
        let sftp = sess.sftp().map_err(|e| e.to_string())?;
        sftp.mkdir(std::path::Path::new(path), 0o755).map_err(|e| e.to_string())
    }

    pub async fn delete_paths(host: &Host, paths: Vec<String>) -> Result<(), String> {
        let sess = establish_ssh_session(host).map_err(|e| e.to_string())?;
        // For deletion, standard RM is more robust for recursive dirs
        for path in paths {
            let cmd = format!("rm -rf '{}'", path.replace("'", "'\\''"));
            let (status, _, err) = crate::ssh::run_remote_script(&sess, &cmd, host.password.as_deref())
                .map_err(|e| e.to_string())?;
            if status != 0 {
                return Err(format!("Delete failed for {}: {}", path, err));
            }
        }
        Ok(())
    }

    pub async fn rename_path(host: &Host, old_path: &str, new_path: &str) -> Result<(), String> {
        let sess = establish_ssh_session(host).map_err(|e| e.to_string())?;
        let sftp = sess.sftp().map_err(|e| e.to_string())?;
        sftp.rename(std::path::Path::new(old_path), std::path::Path::new(new_path), None)
            .map_err(|e| e.to_string())
    }

    pub async fn read_file_preview(host: &Host, path: &str, limit_bytes: usize) -> Result<String, String> {
        let sess = establish_ssh_session(host).map_err(|e| e.to_string())?;
        let sftp = sess.sftp().map_err(|e| e.to_string())?;
        let mut file = sftp.open(std::path::Path::new(path)).map_err(|e| e.to_string())?;
        
        let mut buffer = vec![0; limit_bytes];
        use std::io::Read;
        let n = file.read(&mut buffer).map_err(|e| e.to_string())?;
        buffer.truncate(n);

        Ok(String::from_utf8_lossy(&buffer).to_string())
    }

    pub async fn initialize_volume(host: &Host, _volume_id: &str, device_path: &str, mount_point: &str) -> Result<(), String> {
        let sess = establish_ssh_session(host).map_err(|e| e.to_string())?;
        
        let user = &host.username;
        let script = format!(r#"
set -e
DEVICE="{device_path}"
MOUNT_POINT="{mount_point}"

# Ensure directory exists
mkdir -p "$MOUNT_POINT"

# Check if already mounted
if mountpoint -q "$MOUNT_POINT"; then
    echo "Already mounted"
else
    # Check for filesystem
    FS_TYPE=$(lsblk -f "$DEVICE" -n -o FSTYPE | tr -d '[:space:]' || true)
    
    if [ -z "$FS_TYPE" ]; then
        echo "No filesystem detected on $DEVICE. Formatting as ext4..."
        mkfs.ext4 -F "$DEVICE"
    else
        echo "Detected $FS_TYPE on $DEVICE. Skipping format - your data is safe."
    fi
    
    echo "Mounting $DEVICE to $MOUNT_POINT..."
    mount "$DEVICE" "$MOUNT_POINT"
    
    # Persistence in fstab
    if ! grep -q "$DEVICE" /etc/fstab; then
        echo "Adding $DEVICE to /etc/fstab"
        echo "$DEVICE $MOUNT_POINT ext4 defaults 0 0" >> /etc/fstab
    fi
fi

# Ensure correct permissions for the SSH user
echo "Setting ownership of $MOUNT_POINT to {user}..."
chown -R {user}:{user} "$MOUNT_POINT"
chmod 775 "$MOUNT_POINT"
"#);

        let (status, _, err) = crate::ssh::run_remote_script(&sess, &script, host.password.as_deref())
            .map_err(|e| e.to_string())?;
            
        if status != 0 {
            return Err(format!("Initialization failed: {}", err));
        }
        
        Ok(())
    }

    // ─── Axum Handlers for Streaming ───────────────────────────────────────────

    pub async fn upload_handler(
        Extension(pool): Extension<Pool<DuckdbConnectionManager>>,
        mut multipart: Multipart,
    ) -> impl IntoResponse {
        // Authenticate manually as this isn't a server fn
        // Skip for now to focus on logic, but in prod we need cookie extraction
        
        let mut volume_id = String::new();
        let mut sub_path = String::new();

        while let Ok(Some(mut field)) = multipart.next_field().await {
            let name = field.name().unwrap_or_default().to_string();
            match name.as_str() {
                "volume_id" => volume_id = field.text().await.unwrap_or_default(),
                "path" => sub_path = field.text().await.unwrap_or_default(),
                "file" => {
                    let filename = field.file_name().unwrap_or("upload").to_string();
                    let (host, _) = match find_active_host_for_volume(pool.clone(), &volume_id).await {
                        Ok(h) => h,
                        Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
                    };
                    let root = match volume_id.as_str() {
                        "cloudlab-isos" => "/mnt/isos",
                        "llm-models" => "/mnt/llm-models",
                        _ => return (StatusCode::BAD_REQUEST, "Invalid volume").into_response(),
                    };
                    let full_path = format!("{}/{}/{}", root, sub_path.trim_matches('/'), filename).replace("//", "/");

                    let res = async {
                        let sess = establish_ssh_session(&host).map_err(|e| e.to_string())?;
                        let sftp = sess.sftp().map_err(|e| e.to_string())?;
                        let mut remote_file = sftp.create(std::path::Path::new(&full_path)).map_err(|e| e.to_string())?;

                        while let Some(chunk) = field.chunk().await.map_err(|e: axum::extract::multipart::MultipartError| e.to_string())? {
                            let chunk_vec = chunk.to_vec();
                            remote_file = tokio::task::spawn_blocking(move || {
                                let mut f = remote_file;
                                f.write_all(&chunk_vec).map(|_| f)
                            }).await.map_err(|e| e.to_string())?
                            .map_err(|e| e.to_string())?;
                        }
                        Ok::<(), String>(())
                    }.await;

                    return match res {
                        Ok(_) => StatusCode::OK.into_response(),
                        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, format!("Upload failed: {}", e)).into_response(),
                    };
                }
                _ => {}
            }
        }

        (StatusCode::BAD_REQUEST, "No file provided").into_response()
    }

    pub async fn download_handler(
        Extension(pool): Extension<Pool<DuckdbConnectionManager>>,
        Query(params): Query<AccessParams>,
    ) -> impl IntoResponse {
        let (host, _) = match find_active_host_for_volume(pool, &params.volume_id).await {
            Ok(h) => h,
            Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
        };

        let root = match params.volume_id.as_str() {
            "cloudlab-isos" => "/mnt/isos",
            "llm-models" => "/mnt/llm-models",
            _ => return (StatusCode::BAD_REQUEST, "Invalid volume").into_response(),
        };

        let full_path = format!("{}/{}", root, params.path.trim_start_matches('/'));
        let filename = std::path::Path::new(&full_path).file_name()
            .and_then(|n| n.to_str()).unwrap_or("file").to_string();

        let (tx, rx) = tokio::sync::mpsc::channel::<Result<axum::body::Bytes, std::io::Error>>(10);
        
        tokio::task::spawn_blocking(move || {
            use std::io::Read;
            let res = (|| -> Result<(), String> {
                let sess = establish_ssh_session(&host).map_err(|e| e.to_string())?;
                let sftp = sess.sftp().map_err(|e| e.to_string())?;
                let mut file = sftp.open(std::path::Path::new(&full_path)).map_err(|e| e.to_string())?;
                
                let mut buffer = [0u8; 64 * 1024]; // 64KB chunks
                loop {
                    let n = file.read(&mut buffer).map_err(|e| e.to_string())?;
                    if n == 0 { break; }
                    if tx.blocking_send(Ok(axum::body::Bytes::copy_from_slice(&buffer[..n]))).is_err() {
                        break;
                    }
                }
                Ok(())
            })();
            if let Err(e) = res {
                let _ = tx.blocking_send(Err(std::io::Error::new(std::io::ErrorKind::Other, e)));
            }
        });

        let body = Body::from_stream(tokio_stream::wrappers::ReceiverStream::new(rx));
        
        Response::builder()
            .header(header::CONTENT_TYPE, "application/octet-stream")
            .header(header::CONTENT_DISPOSITION, format!("attachment; filename=\"{}\"", filename))
            .body(body)
            .unwrap()
            .into_response()
    }
}
