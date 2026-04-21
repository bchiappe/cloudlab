use leptos::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Default)]
pub struct Host {
    pub id: String,
    pub name: String,
    pub address: String,
    pub port: i32,
    pub username: String,
    pub auth_method: String,
    pub password: Option<String>,
    pub ssh_key: Option<String>,
    pub ssh_public_key: Option<String>,
    pub ssh_passphrase: Option<String>,
    pub notes: String,
    pub status: String,
    pub zfs_pool_size_gb: i32,
    pub storage_device: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DriveInfo {
    pub name: String,
    pub model: Option<String>,
    pub serial: Option<String>,
    pub vendor: Option<String>,
    pub tran: Option<String>,
    pub size: String,
    pub fstype: Option<String>,
    pub type_name: String,
}

#[cfg(feature = "ssr")]
use duckdb::params;

#[cfg(feature = "ssr")]
use crate::auth::srv_err;

#[cfg(feature = "ssr")]
pub static GLOBAL_STATS: once_cell::sync::Lazy<dashmap::DashMap<String, HostStats>> = 
    once_cell::sync::Lazy::new(dashmap::DashMap::new);

// ─── List ────────────────────────────────────────────────────────────────────

#[cfg(feature = "ssr")]
pub async fn list_hosts_internal(pool: r2d2::Pool<duckdb::DuckdbConnectionManager>) -> Result<Vec<Host>, ServerFnError> {
    let hosts = tokio::task::spawn_blocking(move || -> Result<Vec<Host>, ServerFnError> {
        let conn = pool.get().map_err(srv_err)?;
        let mut stmt = conn
            .prepare(
                "SELECT id, name, address, port, username, auth_method, password, ssh_key, ssh_public_key, ssh_passphrase, notes, status, zfs_pool_size_gb, storage_device \
                 FROM hosts ORDER BY name ASC;",
            )
            .map_err(srv_err)?;
        let mut rows = stmt.query(params![]).map_err(srv_err)?;
        let mut hosts = Vec::new();
        while let Some(row) = rows.next().map_err(srv_err)? {
            hosts.push(Host {
                id: row.get::<_, String>(0).map_err(srv_err)?,
                name: row.get::<_, String>(1).map_err(srv_err)?,
                address: row.get::<_, String>(2).map_err(srv_err)?,
                port: row.get::<_, i32>(3).map_err(srv_err)?,
                username: row.get::<_, String>(4).map_err(srv_err)?,
                auth_method: row.get::<_, String>(5).map_err(srv_err)?,
                password: row.get::<_, Option<String>>(6).map_err(srv_err)?,
                ssh_key: row.get::<_, Option<String>>(7).map_err(srv_err)?,
                ssh_public_key: row.get::<_, Option<String>>(8).map_err(srv_err)?,
                ssh_passphrase: row.get::<_, Option<String>>(9).map_err(srv_err)?,
                notes: row.get::<_, String>(10).map_err(srv_err)?,
                status: row.get::<_, String>(11).map_err(srv_err)?,
                zfs_pool_size_gb: row.get::<_, i32>(12).map_err(srv_err)?,
                storage_device: row.get::<_, Option<String>>(13).map_err(srv_err)?,
            });
        }
        Ok(hosts)
    })
    .await
    .map_err(srv_err)??;

    Ok(hosts)
}

#[server(ListHosts, "/api")]
pub async fn list_hosts() -> Result<Vec<Host>, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        crate::auth::require_role(crate::auth::UserRole::Viewer).await?;
        use axum::extract::Extension;
        use duckdb::DuckdbConnectionManager;
        use leptos_axum::extract;
        use r2d2::Pool;

        let Extension(pool) = extract::<Extension<Pool<DuckdbConnectionManager>>>()
            .await
            .map_err(|_| srv_err("Failed to extract DB pool"))?;

        list_hosts_internal(pool).await
    }
    #[cfg(not(feature = "ssr"))]
    { unreachable!() }
}


// ─── Create ──────────────────────────────────────────────────────────────────

#[server(CreateHost, "/api")]
pub async fn create_host(
    name: String,
    address: String,
    port: i32,
    username: String,
    auth_method: String,
    password: Option<String>,
    ssh_key: Option<String>,
    ssh_public_key: Option<String>,
    ssh_passphrase: Option<String>,
    notes: String,
    zfs_pool_size_gb: i32,
    storage_device: Option<String>,
) -> Result<String, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        crate::auth::require_role(crate::auth::UserRole::Operator).await?;
        use axum::extract::Extension;
        use duckdb::DuckdbConnectionManager;
        use leptos_axum::extract;
        use r2d2::Pool;

        let Extension(pool) = extract::<Extension<Pool<DuckdbConnectionManager>>>()
            .await
            .map_err(|_| srv_err("Failed to extract DB pool"))?;

        let pool_for_setup = pool.clone();
        let id = uuid::Uuid::new_v4().to_string();
        let id_clone = id.clone();
        tokio::task::spawn_blocking(move || -> Result<(), ServerFnError> {
            let conn = pool.get().map_err(srv_err)?;
            conn.execute(
                "INSERT INTO hosts (id, name, address, port, username, auth_method, password, ssh_key, ssh_public_key, ssh_passphrase, notes, status, zfs_pool_size_gb, storage_device) \
                 VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, 'unknown', ?, ?);",
                params![id_clone, name, address, port, username, auth_method, password, ssh_key, ssh_public_key, ssh_passphrase, notes, zfs_pool_size_gb, storage_device],
            ).map_err(srv_err)?;
            Ok(())
        }).await.map_err(srv_err)??;

        // Auto-trigger setup
        let id_for_setup = id.clone();
        tokio::spawn(async move {
            let _ = setup_host_task(pool_for_setup, id_for_setup).await;
        });

        Ok(id)
    }
    #[cfg(not(feature = "ssr"))]
    { unreachable!() }
}

#[cfg(feature = "ssr")]
thread_local! {
    static CONTROLLER_CACHE: std::cell::RefCell<Option<(crate::RequestId, Option<Host>)>> = std::cell::RefCell::new(None);
}

#[cfg(feature = "ssr")]
pub async fn get_controller_host_internal(pool: r2d2::Pool<duckdb::DuckdbConnectionManager>) -> Result<Option<Host>, ServerFnError> {
    let hosts = list_hosts_internal(pool).await?;
    if let Some(host) = hosts.into_iter().find(|h| h.name == "cloudlab-1") {
        return Ok(Some(host));
    }
    Ok(None)
}

#[cfg(feature = "ssr")]
pub async fn get_controller_host(pool: r2d2::Pool<duckdb::DuckdbConnectionManager>) -> Result<Option<Host>, ServerFnError> {
    use axum::extract::Extension;
    use leptos_axum::extract;

    let request_id = extract::<Extension<crate::RequestId>>().await
        .map(|e| e.0.clone())
        .unwrap_or(crate::RequestId("none".into()));

    let cached = CONTROLLER_CACHE.with(|c| {
        let borrow = c.borrow();
        if let Some((rid, h)) = borrow.as_ref() {
            if rid == &request_id {
                return Some(h.clone());
            }
        }
        None
    });

    if cached.is_some() {
        return Ok(cached.unwrap());
    }

    let host = get_controller_host_internal(pool).await?;

    // Update cache
    let h_clone = host.clone();
    CONTROLLER_CACHE.with(|c| {
        *c.borrow_mut() = Some((request_id, h_clone));
    });

    Ok(host)
}

// ─── Update ──────────────────────────────────────────────────────────────────

#[server(UpdateHost, "/api")]
pub async fn update_host(
    id: String,
    name: String,
    address: String,
    port: i32,
    username: String,
    auth_method: String,
    password: Option<String>,
    ssh_key: Option<String>,
    ssh_public_key: Option<String>,
    ssh_passphrase: Option<String>,
    notes: String,
    zfs_pool_size_gb: i32,
    storage_device: Option<String>,
) -> Result<(), ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        crate::auth::require_role(crate::auth::UserRole::Operator).await?;
        use axum::extract::Extension;
        use duckdb::DuckdbConnectionManager;
        use leptos_axum::extract;
        use r2d2::Pool;

        let Extension(pool) = extract::<Extension<Pool<DuckdbConnectionManager>>>()
            .await
            .map_err(|_| srv_err("Failed to extract DB pool"))?;

        tokio::task::spawn_blocking(move || -> Result<(), ServerFnError> {
            let conn = pool.get().map_err(srv_err)?;
            conn.execute(
                "UPDATE hosts SET name=?, address=?, port=?, username=?, auth_method=?, password=?, ssh_key=?, ssh_public_key=?, ssh_passphrase=?, notes=?, zfs_pool_size_gb=?, storage_device=? \
                 WHERE id=?;",
                params![name, address, port, username, auth_method, password, ssh_key, ssh_public_key, ssh_passphrase, notes, zfs_pool_size_gb, storage_device, id],
            )
            .map_err(srv_err)?;
            Ok(())
        })
        .await
        .map_err(srv_err)??;

        Ok(())
    }
    #[cfg(not(feature = "ssr"))]
    { unreachable!() }
}

// ─── Delete ──────────────────────────────────────────────────────────────────

#[server(DeleteHost, "/api")]
pub async fn delete_host(id: String) -> Result<(), ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        crate::auth::require_role(crate::auth::UserRole::Operator).await?;
        use axum::extract::Extension;
        use duckdb::DuckdbConnectionManager;
        use leptos_axum::extract;
        use r2d2::Pool;

        let Extension(pool) = extract::<Extension<Pool<DuckdbConnectionManager>>>()
            .await
            .map_err(|_| srv_err("Failed to extract DB pool"))?;

        tokio::task::spawn_blocking(move || -> Result<(), ServerFnError> {
            let conn = pool.get().map_err(srv_err)?;
            conn.execute("DELETE FROM hosts WHERE id=?;", params![id])
                .map_err(srv_err)?;
            Ok(())
        })
        .await
        .map_err(srv_err)??;

        Ok(())
    }
    #[cfg(not(feature = "ssr"))]
    { unreachable!() }
}

#[server(ListHostDrives, "/api")]
pub async fn list_host_drives(host_id: String) -> Result<Vec<DriveInfo>, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        crate::auth::require_role(crate::auth::UserRole::Operator).await?;
        use axum::extract::Extension;
        use duckdb::DuckdbConnectionManager;
        use leptos_axum::extract;
        use r2d2::Pool;
        use std::io::Read;

        let Extension(pool) = extract::<Extension<Pool<DuckdbConnectionManager>>>()
            .await
            .map_err(|_| srv_err("Failed to extract DB pool"))?;

        let drives = tokio::task::spawn_blocking(move || -> Result<Vec<DriveInfo>, ServerFnError> {
            let conn = pool.get().map_err(srv_err)?;
            let sess = get_host_session_blocking(&conn, &host_id)?;
            
            let mut ch = sess.channel_session().map_err(srv_err)?;
            ch.exec("sudo lsblk -J -o NAME,MODEL,SIZE,TYPE,FSTYPE,SERIAL,VENDOR,TRAN").map_err(srv_err)?;
            let mut output = String::new();
            ch.read_to_string(&mut output).map_err(srv_err)?;
            
            println!("DEBUG: Raw lsblk output for host {}:\n{}", host_id, output);

            let json: serde_json::Value = serde_json::from_str(&output).map_err(srv_err)?;
            let mut drives = Vec::new();
            
            if let Some(blockdevices) = json["blockdevices"].as_array() {
                for dev in blockdevices {
                    let name = dev["name"].as_str().unwrap_or("unknown").to_string();
                    let model = dev["model"].as_str().map(|s| s.trim().to_string());
                    let serial = dev["serial"].as_str().map(|s| s.trim().to_string());
                    let vendor = dev["vendor"].as_str().map(|s| s.trim().to_string());
                    let tran = dev["tran"].as_str().map(|s| s.trim().to_string());
                    let size = dev["size"].as_str().unwrap_or("unknown").to_string();
                    let fstype = dev["fstype"].as_str().map(|s| s.to_string());
                    let type_name = dev["type"].as_str().unwrap_or("unknown").to_string();
                    
                    if type_name == "disk" {
                        drives.push(DriveInfo {
                            name: format!("/dev/{}", name),
                            model,
                            serial,
                            vendor,
                            tran,
                            size,
                            fstype,
                            type_name,
                        });
                    }
                }
            }
            
            Ok(drives)
        }).await.map_err(srv_err)??;

        Ok(drives)
    }
    #[cfg(not(feature = "ssr"))]
    { unreachable!() }
}

// ─── Test Connection ──────────────────────────────────────────────────────────

#[server(TestHostConnection, "/api")]
pub async fn test_host_connection(id: String) -> Result<String, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        crate::auth::require_role(crate::auth::UserRole::Operator).await?;
        use axum::extract::Extension;
        use duckdb::DuckdbConnectionManager;
        use leptos_axum::extract;
        use r2d2::Pool;
        use std::time::Duration;

        let Extension(pool) = extract::<Extension<Pool<DuckdbConnectionManager>>>()
            .await
            .map_err(|_| srv_err("Failed to extract DB pool"))?;

        // Fetch address + port from DB
        let pool_clone = pool.clone();
        let id_clone = id.clone();
        let (address, port) =
            tokio::task::spawn_blocking(move || -> Result<(String, i32), ServerFnError> {
                let conn = pool_clone.get().map_err(srv_err)?;
                let mut stmt = conn
                    .prepare("SELECT address, port FROM hosts WHERE id=?;")
                    .map_err(srv_err)?;
                stmt.query_row(params![id_clone], |row| {
                    Ok((row.get::<_, String>(0)?, row.get::<_, i32>(1)?))
                })
                .map_err(srv_err)
            })
            .await
            .map_err(srv_err)??;

        // TCP connectivity check with 5 s timeout
        let addr = format!("{}:{}", address, port);
        let new_status = match tokio::time::timeout(
            Duration::from_secs(5),
            tokio::net::TcpStream::connect(&addr),
        )
        .await
        {
            Ok(Ok(_)) => "online".to_string(),
            _ => "offline".to_string(),
        };

        // Persist updated status
        let status_clone = new_status.clone();
        tokio::task::spawn_blocking(move || -> Result<(), ServerFnError> {
            let conn = pool.get().map_err(srv_err)?;
            conn.execute(
                "UPDATE hosts SET status=? WHERE id=?;",
                params![status_clone, id],
            )
            .map_err(srv_err)?;
            Ok(())
        })
        .await
        .map_err(srv_err)??;

        Ok(new_status)
    }
    #[cfg(not(feature = "ssr"))]
    { unreachable!() }
}

// ─── Verify Dependencies ──────────────────────────────────────────────────────

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DependencyReport {
    pub qemu_installed: bool,
    pub docker_installed: bool,
    pub nvidia_smi_available: bool,
    pub nvidia_runtime_configured: bool,
}

#[server(VerifyHostDependencies, "/api")]
pub async fn verify_host_dependencies(id: String) -> Result<DependencyReport, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        crate::auth::require_role(crate::auth::UserRole::Operator).await?;
        use ax_ext::Extension;
        use duckdb::DuckdbConnectionManager;
        use leptos_axum::extract;
        use r2d2::Pool;
        use axum::extract as ax_ext;

        let Extension(pool) = extract::<Extension<Pool<DuckdbConnectionManager>>>()
            .await
            .map_err(|_| srv_err("Failed to extract DB pool"))?;

        let report = tokio::task::spawn_blocking(move || -> Result<DependencyReport, ServerFnError> {
            let conn = pool.get().map_err(srv_err)?;
            let sess = crate::hosts::get_host_session_blocking(&conn, &id)?;

            let (v_status, _, _) = crate::ssh::run_remote_script(&sess, "which qemu-system-x86_64", None)?;
            let qemu = v_status == 0;

            let (d_status, _, _) = crate::ssh::run_remote_script(&sess, "which docker", None)?;
            let docker = d_status == 0;

            let (n_status, _, _) = crate::ssh::run_remote_script(&sess, "nvidia-smi", None)?;
            let nvidia_smi = n_status == 0;

            let (_r_status, r_stdout, _) = crate::ssh::run_remote_script(&sess, "docker info | grep -i runtime", None)?;
            let nvidia_runtime = r_stdout.to_lowercase().contains("nvidia");

            Ok(DependencyReport { 
                qemu_installed: qemu, 
                docker_installed: docker,
                nvidia_smi_available: nvidia_smi,
                nvidia_runtime_configured: nvidia_runtime,
            })
        }).await.map_err(srv_err)??;

        Ok(report)
    }
    #[cfg(not(feature = "ssr"))]
    { unreachable!() }
}

// ─── Setup Host ──────────────────────────────────────────────────────────────

#[server(SetupHost, "/api")]
pub async fn setup_host(id: String) -> Result<(), ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        crate::auth::require_role(crate::auth::UserRole::Operator).await?;
        use axum::extract::Extension;
        use leptos_axum::extract;
        use duckdb::DuckdbConnectionManager;
        use r2d2::Pool;

        let Extension(pool) = extract::<Extension<Pool<DuckdbConnectionManager>>>()
            .await
            .map_err(|_| srv_err("Failed to extract DB pool"))?;

        setup_host_task(pool, id).await
    }
    #[cfg(not(feature = "ssr"))]
    { unreachable!() }
}

#[cfg(feature = "ssr")]
pub async fn setup_host_task(
    pool: r2d2::Pool<duckdb::DuckdbConnectionManager>,
    id: String,
) -> Result<(), ServerFnError> {
    use duckdb::params;
    use std::net::TcpStream;
    use ssh2::Session;
    use crate::ssh::{get_cloudlab_ssh_key};

    // 1. Get host info
    let id_query = id.clone();
    let pool_first_task = pool.clone();
    let (node_name, address, port, user, method, pw, key, pub_key, passphrase, zfs_pool_size, storage_device) = tokio::task::spawn_blocking(move || -> Result<(String, String, i32, String, String, String, String, String, String, i32, Option<String>), ServerFnError> {
        let conn = pool_first_task.get().map_err(srv_err)?;
        let mut stmt = conn.prepare("SELECT name, address, port, username, auth_method, password, ssh_key, ssh_public_key, ssh_passphrase, zfs_pool_size_gb, storage_device FROM hosts WHERE id=?;").map_err(srv_err)?;
        stmt.query_row(params![id_query], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, i32>(2)?,
                row.get::<_, String>(3)?,
                row.get::<_, String>(4)?,
                row.get::<_, Option<String>>(5)?.unwrap_or_default(),
                row.get::<_, Option<String>>(6)?.unwrap_or_default(),
                row.get::<_, Option<String>>(7)?.unwrap_or_default(),
                row.get::<_, Option<String>>(8)?.unwrap_or_default(),
                row.get::<_, i32>(9)?,
                row.get::<_, Option<String>>(10)?,
            ))
        }).map_err(srv_err)
    }).await.map_err(srv_err)??;

    let pool_first_check = pool.clone();
    let id_for_first = id.clone();
    let is_controller = tokio::task::spawn_blocking(move || -> bool {
        let conn = pool_first_check.get().ok();
        if let Some(conn) = conn {
            let stmt = conn.prepare("SELECT id FROM hosts ORDER BY rowid ASC LIMIT 1;").ok();
            if let Some(mut stmt) = stmt {
                let first_id: Option<String> = stmt.query_row(params![], |r| r.get(0)).ok();
                return first_id == Some(id_for_first);
            }
        }
        false
    }).await.unwrap_or(false);

    let job_id = crate::jobs::create_job(pool.clone(), format!("Host Setup: {}", address)).await?;
    crate::jobs::update_job(pool.clone(), job_id.clone(), "running".into(), 5).await?;
    crate::jobs::add_job_log(pool.clone(), job_id.clone(), "Starting host onboarding...".into()).await?;

    // 2. Get CloudLab Master Key
    let cloudlab_keys = match get_cloudlab_ssh_key(pool.clone()).await {
        Ok(k) => k,
        Err(e) => {
            crate::jobs::update_job(pool.clone(), job_id.clone(), "failed".into(), 10).await?;
            return Err(e);
        }
    };

    // 3. Setup Logic
    let pool_inner = pool.clone();
    let address_for_setup = address.clone();
    let job_id_log = job_id.clone();
    let pool_log = pool.clone();
    let node_name_inner = node_name.clone();
    let address_inner = address.clone();
    let res = tokio::task::spawn_blocking(move || -> Result<(), ServerFnError> {
        let _ = crate::jobs::add_job_log(pool_log.clone(), job_id_log.clone(), "Installing virtualization and storage dependencies...".into());
        let tcp = TcpStream::connect(format!("{}:{}", address_for_setup, port)).map_err(srv_err)?;
        let mut sess = Session::new().map_err(srv_err)?;
        sess.set_tcp_stream(tcp);
        sess.handshake().map_err(srv_err)?;

        let final_pub_key = if pub_key.is_empty() && !key.is_empty() {
             crate::ssh::extract_public_key(&key).unwrap_or_default()
        } else {
            pub_key
        };

        if method == "password" {
            sess.userauth_password(&user, &pw).map_err(srv_err)?;
        } else {
            let pk = if final_pub_key.is_empty() { None } else { Some(final_pub_key.as_str()) };
            let p_phrase = if passphrase.is_empty() { None } else { Some(passphrase.as_str()) };
            sess.userauth_pubkey_memory(&user, pk, &key, p_phrase).map_err(srv_err)?;
        }

        let extra_packages = if is_controller { "linstor-controller linstor-client" } else { "" };
        let script = format!(
            "set -ex\n\
             if [ \"$(id -u)\" -ne 0 ]; then echo \"This script must be run as root\"; exit 1; fi\n\
             useradd -m -s /bin/bash cloudlab || true\n\
             mkdir -p /home/cloudlab/.ssh\n\
             echo '{}' > /home/cloudlab/.ssh/authorized_keys\n\
             chown -R cloudlab:cloudlab /home/cloudlab/.ssh\n\
             chmod 700 /home/cloudlab/.ssh\n\
             chmod 600 /home/cloudlab/.ssh/authorized_keys\n\
             echo 'cloudlab ALL=(ALL) NOPASSWD:ALL' > /etc/sudoers.d/cloudlab\n\
             apt-get update\n\
             apt-get install -y software-properties-common\n\
             add-apt-repository -y ppa:linbit/linbit-drbd9-stack\n\
             apt-get update\n\
             apt-get install -y zfsutils-linux lvm2 drbd-dkms drbd-utils linstor-satellite qemu-kvm libvirt-daemon-system libvirt-clients bridge-utils docker.io curl ca-certificates lsof psmisc ubuntu-drivers-common novnc websockify {}\n\
             usermod -aG docker cloudlab\n\
             usermod -aG libvirt cloudlab\n\
             systemctl enable --now docker\n\
             systemctl enable --now libvirtd\n\
\n\
             # Bridge Auto-detection\n\
             echo \"Detecting network bridge...\"\n\
             IF_BRIDGE=$(brctl show | awk 'NR>1 {{print $1}}' | grep -E \"br0|virbr0\" | head -n 1)\n\
             if [ -z \"$IF_BRIDGE\" ]; then\n\
                IF_BRIDGE=$(brctl show | awk 'NR>1 {{print $1}}' | head -n 1)\n\
             fi\n\
             if [ -n \"$IF_BRIDGE\" ]; then\n\
                echo \"AUTO_DETECTED_BRIDGE: $IF_BRIDGE\"\n\
             else\n\
                echo \"WARNING: No network bridge detected. KVM networking may fail.\"\n\
             fi\n\
             \n\
             # NVIDIA Driver & Container Toolkit Setup\n\
             if lspci | grep -i nvidia; then\n\
                if ! command -v nvidia-smi &> /dev/null; then\n\
                   echo \"NVIDIA GPU detected but drivers missing. Installing recommended drivers...\"\n\
                   ubuntu-drivers autoinstall || true\n\
                   echo \"REBOOT_REQUIRED: NVIDIA drivers installed. Please reboot the host.\"\n\
                fi\n\
                echo \"NVIDIA GPU detected. Installing/Configuring NVIDIA Container Toolkit...\"\n\
                curl -fsSL https://nvidia.github.io/libnvidia-container/gpgkey | sudo gpg --dearmor -o /usr/share/keyrings/nvidia-container-toolkit-keyring.gpg || true\n\
                curl -s -L https://nvidia.github.io/libnvidia-container/stable/deb/nvidia-container-toolkit.list | \\\n\
                  sed 's#deb https://#deb [signed-by=/usr/share/keyrings/nvidia-container-toolkit-keyring.gpg] https://#g' | \\\n\
                  sudo tee /etc/apt/sources.list.d/nvidia-container-toolkit.list || true\n\
                apt-get update\n\
                apt-get install -y nvidia-container-toolkit nvidia-cuda-toolkit libvulkan1 vulkan-tools build-essential cmake git pkg-config libvulkan-dev\\n\\
                if ! command -v rustc &> /dev/null; then\\n\\
                    echo \"Installing Rust toolchain...\"\\n\\
                    curl -fsSL https://sh.rustup.rs | sh -s -- -y\\n\\
                    source $HOME/.cargo/env\\n\\
                fi\\n\\
                echo \"--- Host Library Check ---\"\\n\\
                ldconfig -p | grep -E \"cublas|cudart|vulkan\" || true\\n\\
                \n\
                # Deep Fix for Docker Runtime\n\
                echo \"Configuring NVIDIA runtime for Docker...\"\n\
                nvidia-ctk runtime configure --runtime=docker\n\
                \n\
                # Ensure the runtime is actually in daemon.json\n\
                if ! grep -q \"nvidia\" /etc/docker/daemon.json 2>/dev/null; then\n\
                   echo \"Manual injection of NVIDIA runtime into daemon.json...\"\n\
                   mkdir -p /etc/docker\n\
                   echo '{{\"runtimes\":{{\"nvidia\":{{\"path\":\"nvidia-container-runtime\",\"runtimeArgs\":[]}}}}}}' > /etc/docker/daemon.json\n\
                fi\n\
                \n\
                systemctl restart docker\n\
                echo \"NVIDIA runtime configuration complete.\"\n\
             fi\n\
             \n\
             echo \"PROVISIONING_START: Handling Storage Pool\"\n\
             DEVICE=\"{}\"\n\
             \n\
             # Aggressively stop services and sockets to release potential locks on the pool\n\
             echo \"Stopping services and sockets (LINSTOR, Docker, Libvirt, DRBD, ZED) to release storage pool...\"\n\
             systemctl stop linstor-satellite linstor-controller || true\n\
             systemctl stop docker.socket docker || true\n\
             # Stop all modular libvirt daemons and their sockets\n\
             systemctl stop 'virt*' || true\n\
             systemctl stop virtlockd.socket virtlogd.socket || true\n\
             systemctl stop libvirtd.socket libvirtd-ro.socket libvirtd-admin.socket libvirtd || true\n\
             systemctl stop zfs-zed || true\n\
             \n\
             # Take down all DRBD resources which are likely backed by ZFS ZVols\n\
             drbdadm down all || true\n\
             udevadm settle || true\n\
             \n\
             if [ -n \"$DEVICE\" ]; then\n\
                echo \"Using raw storage device: $DEVICE\"\n\
                DEV_SHORT=$(echo \"$DEVICE\" | sed 's|/dev/||')\n\
                if sudo zpool list cloudlab_zpool >/dev/null 2>&1; then\n\
                   if ! sudo zpool list -v cloudlab_zpool | grep -q \"$DEV_SHORT\"; then\n\
                      echo \"Device mismatch detected. Attempting to release ZVols and destroy old pool...\"\n\
                      \n\
                      # Specifically unmount ZVols that might be mounted (like /mnt/llm-models)\n\
                      for zd in /dev/zd*; do\n\
                         if [ -b \"$zd\" ]; then\n\
                            echo \"Unmounting ZVol $zd...\"\n\
                            sudo umount -f \"$zd\" || true\n\
                         fi\n\
                      done\n\
                      sudo umount -f /mnt/llm-models || true\n\
                      sudo umount -f /mnt/isos || true\n\
                      \n\
                      sudo zfs unmount -a || true\n\
                      # Lazy unmount fallback\n\
                      sudo umount -fl /var/lib/linstor/cloudlab_zpool || true\n\
                      \n\
                      udevadm settle || true\n\
                      # Try to destroy or export the old pool\n\
                      if sudo zpool list cloudlab_zpool >/dev/null 2>&1; then\n\
                         sudo zpool destroy -f cloudlab_zpool || sudo zpool export -f cloudlab_zpool || true\n\
                      fi\n\
                      udevadm settle || true\n\
                   fi\n\
                fi\n\
                if ! sudo zpool list cloudlab_zpool >/dev/null 2>&1; then\n\
                   echo \"Creating ZFS pool on $DEVICE...\"\n\
                   sudo zpool create -f cloudlab_zpool \"$DEVICE\"\n\
                   udevadm settle || true\n\
                fi\n\
             else\n\
                echo \"Using file-backed storage pool\"\n\
                if sudo zpool list cloudlab_zpool >/dev/null 2>&1 && ! sudo zpool list -v cloudlab_zpool | grep -q \".img\"; then\n\
                   echo \"Physical pool detected but file requested. Destroying to migrate to file...\"\n\
                   sudo zfs unmount -a || true\n\
                   sudo zpool export -f cloudlab_zpool || true\n\
                   sudo zpool destroy -f cloudlab_zpool\n\
                   udevadm settle || true\n\
                fi\n\
                mkdir -p /var/lib/linstor\n\
                [[ ! -f /var/lib/linstor/cloudlab_zpool.img ]] && truncate -s {}G /var/lib/linstor/cloudlab_zpool.img || true\n\
                if ! sudo zpool list cloudlab_zpool >/dev/null 2>&1; then\n\
                   echo \"Creating file-backed ZFS pool...\"\n\
                   sudo zpool create cloudlab_zpool /var/lib/linstor/cloudlab_zpool.img\n\
                   udevadm settle || true\n\
                fi\n\
             fi\n\
             \n\
             echo \"Restarting services (LINSTOR, Docker, Libvirt)...\"\n\
             systemctl restart docker || true\n\
             systemctl restart libvirtd || true\n\
             systemctl restart linstor-satellite || true\n\
             if systemctl list-unit-files | grep -q linstor-controller; then\n\
                systemctl restart linstor-controller || true\n\
             fi\n\
             udevadm settle || true\n\
             \n\
             echo \"PROVISIONING_STEP: Registering with LINSTOR\"\n\
             linstor node create {} {} --node-type SATELLITE || true\n\
             linstor resource-group create default || true\n\
             linstor storage-pool create zfs {} cloudlab_pool cloudlab_zpool || echo \"Linstor Pool Registration Failed (already exists?)\"",
            cloudlab_keys.public_key,
            extra_packages,
            storage_device.clone().unwrap_or_default(),
            zfs_pool_size,
            node_name_inner, address_inner, node_name_inner
        );

        let (status, stdout, stderr) = crate::ssh::run_remote_script(&sess, &script, if method == "password" { Some(&pw) } else { None })?;
        
        leptos::logging::log!("DEBUG: Host setup script finished. Status: {}. Stdout: {}, Stderr: {}", status, stdout, stderr);
        let _ = crate::jobs::add_job_log(pool_log.clone(), job_id_log.clone(), format!("Configuration script finished (exit {}).", status).into());
        
        if status != 0 {
            return Err(srv_err(format!("Host setup failed (exit {}).\nSTDOUT: {}\nSTDERR: {}", status, stdout, stderr)));
        }

        // 4. Update host credentials in DB if we were using initial credentials
        if user != "cloudlab" {
            let conn = pool_inner.get().map_err(srv_err)?;
            conn.execute(
                "UPDATE hosts SET username='cloudlab', auth_method='pubkey', ssh_key=?, ssh_public_key=?, password=NULL WHERE id=?;",
                params![cloudlab_keys.private_key, cloudlab_keys.public_key, id],
            ).map_err(srv_err)?;
        }

        Ok(())
    }).await.map_err(srv_err)?;

    // 3.1 Special case: If this is NOT the controller, we must register it via the controller
    if !is_controller {
        crate::jobs::update_job(pool.clone(), job_id.clone(), "running".into(), 85).await?;
        let controller = get_controller_host(pool.clone()).await?
            .ok_or_else(|| srv_err("No controller host found to register satellite"))?;
        let controller_id = controller.id.clone();
        let pool_for_reg = pool.clone();
        
        let r_res = tokio::task::spawn_blocking(move || -> Result<(), ServerFnError> {
            let conn = pool_for_reg.get().map_err(srv_err)?;
            let controller_sess = crate::hosts::get_host_session_blocking(&conn, &controller_id)?;
            let reg_script = format!(
                "linstor node create {} {} --node-type SATELLITE || true\n\
                 linstor storage-pool create zfs {} cloudlab_pool cloudlab_zpool || true",
                node_name, address, node_name
            );
            let (r_status, r_out, r_err) = crate::ssh::run_remote_script(&controller_sess, &reg_script, None)?;
            if r_status != 0 {
                 return Err(srv_err(format!("Node registration on controller failed (exit {}).\nSTDOUT: {}\nSTDERR: {}", r_status, r_out, r_err)));
            }
            Ok(())
        }).await.map_err(srv_err)?;
        
        if let Err(e) = r_res {
            crate::jobs::update_job(pool.clone(), job_id.clone(), "failed".into(), 90).await?;
            return Err(e);
        }
    }

    match res {
        Ok(_) => {
            crate::jobs::update_job(pool.clone(), job_id.clone(), "completed".into(), 100).await?;
        }
        Err(e) => {
            leptos::logging::log!("DEBUG: Task failed for job {}: {:?}", job_id, e);
            crate::jobs::update_job(pool.clone(), job_id.clone(), "failed".into(), 50).await?;
            return Err(e);
        }
    }

    Ok(())
}

// ─── Real-time Stats ──────────────────────────────────────────────────────────

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct HostStats {
    pub cpu_usage: f64,
    pub mem_usage: f64,
    pub disk_usage: f64,
}

#[server(GetHostStats, "/api")]
pub async fn get_host_stats(id: String) -> Result<HostStats, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        if let Some(stats) = GLOBAL_STATS.get(&id) {
            Ok(stats.clone())
        } else {
            Ok(HostStats { cpu_usage: 0.0, mem_usage: 0.0, disk_usage: 0.0 })
        }
    }
    #[cfg(not(feature = "ssr"))]
    { unreachable!() }
}

#[server(ResizeHostPool, "/api")]
pub async fn resize_host_pool(id: String, new_size_gb: i32) -> Result<(), ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        crate::auth::require_role(crate::auth::UserRole::Operator).await?;
        use axum::extract::Extension;
        use leptos_axum::extract;
        use duckdb::DuckdbConnectionManager;
        use r2d2::Pool;

        let Extension(pool) = extract::<Extension<Pool<DuckdbConnectionManager>>>()
            .await
            .map_err(|_| srv_err("Failed to extract DB pool"))?;

        let job_id = crate::jobs::create_job(pool.clone(), format!("Resize Storage Pool: {} ({}G)", id, new_size_gb)).await?;
        let pool_inner = pool.clone();
        let id_inner = id.clone();
        let name_inner = tokio::task::spawn_blocking({
            let p = pool.clone();
            let id = id.clone();
            move || {
                let conn = p.get().ok()?;
                let mut stmt = conn.prepare("SELECT name, storage_device FROM hosts WHERE id = ?;").ok()?;
                stmt.query_row(duckdb::params![id], |r| Ok((r.get::<_, String>(0)?, r.get::<_, Option<String>>(1)?))).ok()
            }
        }).await.unwrap_or_default().unwrap_or_else(|| ("unknown".into(), None));

        tokio::spawn(async move {
            let pool_for_res = pool_inner.clone();
            let id_for_res = id_inner.clone();
            let job_id_for_res = job_id.clone();

            let res = (async move || -> Result<(), ServerFnError> {
                let conn = pool_for_res.get().map_err(srv_err)?;
                let sess = crate::hosts::get_host_session_blocking(&conn, &id_for_res)?;

                let script = if let Some(dev) = name_inner.1 {
                    format!(
                        "set -ex\n\
                         echo 'Expanding physical storage pool on {}...'\n\
                         sudo zpool set autoexpand=on cloudlab_zpool\n\
                         sudo zpool online -e cloudlab_zpool {}\n\
                         sudo zpool reopen cloudlab_zpool\n\
                         \n\
                         echo 'Refreshing Linstor...'\n\
                         sudo linstor node modify {} || true\n\
                         \n\
                         echo 'Final status check:'\n\
                         sudo zpool list cloudlab_zpool\n\
                         sudo linstor storage-pool list",
                        dev, dev, name_inner.0
                    )
                } else {
                    format!(
                        "set -ex\n\
                         echo 'Checking physical disk space on host...'\n\
                         df -h /var/lib/linstor/\n\
                         \n\
                         echo 'Expanding backing file to {}G...'\n\
                         sudo truncate -s {}G /var/lib/linstor/cloudlab_zpool.img\n\
                         ls -lh /var/lib/linstor/cloudlab_zpool.img\n\
                         \n\
                         echo 'Forcing ZFS expansion...'\n\
                         sudo zpool set autoexpand=on cloudlab_zpool\n\
                         sudo zpool online -e cloudlab_zpool /var/lib/linstor/cloudlab_zpool.img\n\
                         sudo zpool reopen cloudlab_zpool\n\
                         \n\
                         echo 'Refreshing Linstor...'\n\
                         sudo linstor node modify {} || true\n\
                         \n\
                         echo 'Final status check:'\n\
                         sudo zpool list cloudlab_zpool\n\
                         sudo linstor storage-pool list",
                        new_size_gb, new_size_gb, name_inner.0
                    )
                };

                let _ = crate::jobs::add_job_log(pool_for_res.clone(), job_id_for_res.clone(), format!("Expanding backing file to {}G...", new_size_gb)).await;
                let (status, stdout, stderr) = crate::ssh::run_remote_script(&sess, &script, None)?;

                if status != 0 {
                    return Err(srv_err(format!("Resize failed (exit {}).\nSTDOUT: {}\nSTDERR: {}", status, stdout, stderr)));
                }

                let _ = crate::jobs::add_job_log(pool_for_res.clone(), job_id_for_res.clone(), format!("Resize output:\n{}", stdout)).await;

                let _ = crate::jobs::add_job_log(pool_for_res.clone(), job_id_for_res.clone(), "Updating database...".into()).await;
                conn.execute(
                    "UPDATE hosts SET zfs_pool_size_gb = ? WHERE id = ?;",
                    duckdb::params![new_size_gb, id_for_res],
                ).map_err(srv_err)?;

                Ok(())
            })().await;

            match res {
                Ok(_) => {
                    let _ = crate::jobs::update_job(pool_inner.clone(), job_id.clone(), "completed".into(), 100).await;
                }
                Err(e) => {
                    let _ = crate::jobs::update_job(pool_inner.clone(), job_id.clone(), "failed".into(), 0).await;
                    let _ = crate::jobs::add_job_log(pool_inner.clone(), job_id.clone(), format!("Error: {}", e)).await;
                }
            }
        });

        Ok(())
    }
    #[cfg(not(feature = "ssr"))]
    { unreachable!() }
}

#[cfg(feature = "ssr")]
pub fn start_background_monitor(pool: r2d2::Pool<duckdb::DuckdbConnectionManager>) {
    use std::time::Duration;
    use std::net::TcpStream;
    use ssh2::Session;
    use std::io::Read;

    use std::sync::atomic::{AtomicBool, Ordering};
    static MONITOR_RUNNING: AtomicBool = AtomicBool::new(false);

    tokio::task::spawn(async move {
        loop {
            if MONITOR_RUNNING.swap(true, Ordering::SeqCst) {
                // Already running, skip this round
                tokio::time::sleep(Duration::from_secs(5)).await;
                continue;
            }

            let hosts: Vec<Result<(String, String, i32, String, String, String, String, String, String), duckdb::Error>> = {
                let conn = pool.get().ok();
                if let Some(conn) = conn {
                    let stmt = conn.prepare("SELECT id, address, port, username, auth_method, password, ssh_key, ssh_public_key, ssh_passphrase FROM hosts").ok();
                    if let Some(mut stmt) = stmt {
                        let rows = stmt.query_map(duckdb::params![], |row| {
                            Ok((
                                row.get::<_, String>(0)?,
                                row.get::<_, String>(1)?,
                                row.get::<_, i32>(2)?,
                                row.get::<_, String>(3)?,
                                row.get::<_, String>(4)?,
                                row.get::<_, Option<String>>(5)?.unwrap_or_default(),
                                row.get::<_, Option<String>>(6)?.unwrap_or_default(),
                                row.get::<_, Option<String>>(7)?.unwrap_or_default(),
                                row.get::<_, Option<String>>(8)?.unwrap_or_default()
                            ))
                        }).ok();
                        rows.map(|r| r.collect::<Vec<_>>()).unwrap_or_default()
                    } else { vec![] }
                } else { vec![] }
            };

            for host in hosts {
                if let Ok((id, addr, port, user, method, pw, key, pub_key, passphrase)) = host {
                    let id_clone = id.clone();
                    let pool_task = pool.clone();
                    tokio::task::spawn_blocking(move || {
                        let res = (|| -> Result<HostStats, String> {
                            let tcp = TcpStream::connect_timeout(&format!("{}:{}", addr, port).parse().unwrap_or("0.0.0.0:0".parse().unwrap()), Duration::from_secs(5)).map_err(|e| e.to_string())?;
                            tcp.set_read_timeout(Some(Duration::from_secs(5))).ok();
                            let mut sess = Session::new().map_err(|e| e.to_string())?;
                            sess.set_timeout(5000);
                            sess.set_tcp_stream(tcp);
                            sess.handshake().map_err(|e| e.to_string())?;
                            
                            if method == "password" {
                                sess.userauth_password(&user, &pw).map_err(|e| e.to_string())?;
                            } else {
                                let mut pk_str = pub_key.clone();
                                if pk_str.is_empty() && !key.is_empty() {
                                    if let Ok(derived) = crate::ssh::extract_public_key(&key) {
                                        pk_str = derived;
                                        // Self-heal DB
                                        let pool_heal = pool_task.clone();
                                        let id_heal = id_clone.clone();
                                        let derived_clone = pk_str.clone();
                                        tokio::task::spawn_blocking(move || {
                                            if let Ok(conn) = pool_heal.get() {
                                                let _ = conn.execute("UPDATE hosts SET ssh_public_key = ? WHERE id = ?;", duckdb::params![derived_clone, id_heal]);
                                            }
                                        });
                                    }
                                }
                                let pk = if pk_str.is_empty() { None } else { Some(pk_str.as_str()) };
                                let p_phrase = if passphrase.is_empty() { None } else { Some(passphrase.as_str()) };
                                sess.userauth_pubkey_memory(&user, pk, &key, p_phrase).map_err(|e| e.to_string())?;
                            }

                            // CPU
                            let mut ch = sess.channel_session().map_err(|e| e.to_string())?;
                            ch.exec("top -bn1 | grep 'Cpu(s)' | awk '{print $8}'").ok();
                            let mut out = String::new();
                            ch.read_to_string(&mut out).ok();
                            let cpu = 100.0 - out.trim().parse::<f64>().unwrap_or(100.0);

                            // Mem
                            let mut ch = sess.channel_session().map_err(|e| e.to_string())?;
                            ch.exec("free | grep Mem | awk '{print $3/$2 * 100.0}'").ok();
                            let mut out = String::new();
                            ch.read_to_string(&mut out).ok();
                            let mem = out.trim().parse::<f64>().unwrap_or(0.0);

                            // Disk
                            let mut ch = sess.channel_session().map_err(|e| e.to_string())?;
                            ch.exec("df / | tail -1 | awk '{print $5}' | sed 's/%//'").ok();
                            let mut out = String::new();
                            ch.read_to_string(&mut out).ok();
                            let disk = out.trim().parse::<f64>().unwrap_or(0.0);

                            Ok(HostStats { cpu_usage: cpu, mem_usage: mem, disk_usage: disk })
                        })();

                        if let Ok(stats) = res {
                            GLOBAL_STATS.insert(id_clone, stats);
                        }
                    });
                }
            }
            MONITOR_RUNNING.store(false, Ordering::SeqCst);
            tokio::time::sleep(Duration::from_secs(10)).await;
        }
    });
}

#[cfg(feature = "ssr")]
pub fn get_host_session_blocking(
    conn: &duckdb::Connection,
    host_id: &str,
) -> Result<ssh2::Session, ServerFnError> {
    use std::net::TcpStream;
    use ssh2::Session;

    let mut stmt = conn.prepare("SELECT address, port, username, auth_method, password, ssh_key, ssh_passphrase FROM hosts WHERE id=?;").map_err(srv_err)?;
    let (addr, port, user, method, pw, key, passphrase) = stmt.query_row(params![host_id], |row| {
        Ok((
            row.get::<_, String>(0)?,
            row.get::<_, i32>(1)?,
            row.get::<_, String>(2)?,
            row.get::<_, String>(3)?,
            row.get::<_, Option<String>>(4)?.unwrap_or_default(),
            row.get::<_, Option<String>>(5)?.unwrap_or_default(),
            row.get::<_, Option<String>>(6)?.unwrap_or_default(),
        ))
    }).map_err(srv_err)?;

    let tcp = TcpStream::connect(format!("{}:{}", addr, port)).map_err(srv_err)?;
    let mut sess = Session::new().map_err(srv_err)?;
    sess.set_tcp_stream(tcp);
    sess.handshake().map_err(srv_err)?;

    if method == "password" {
        sess.userauth_password(&user, &pw).map_err(srv_err)?;
    } else {
        let p_phrase = if passphrase.is_empty() { None } else { Some(passphrase.as_str()) };
        sess.userauth_pubkey_memory(&user, None, &key, p_phrase).map_err(srv_err)?;
    }

    Ok(sess)
}

#[cfg(feature = "ssr")]
pub fn establish_ssh_session(host: &Host) -> Result<ssh2::Session, ServerFnError> {
    use std::net::TcpStream;
    use ssh2::Session;

    let addr = format!("{}:{}", host.address, host.port);
    let tcp = TcpStream::connect(&addr).map_err(srv_err)?;
    let mut sess = Session::new().map_err(srv_err)?;
    sess.set_tcp_stream(tcp);
    sess.handshake().map_err(srv_err)?;

    if host.auth_method == "password" {
        let pw = host.password.as_deref().unwrap_or_default();
        sess.userauth_password(&host.username, pw).map_err(srv_err)?;
    } else {
        let key = host.ssh_key.as_deref().unwrap_or_default();
        let pub_key = host.ssh_public_key.as_deref();
        let passphrase = host.ssh_passphrase.as_deref();
        sess.userauth_pubkey_memory(&host.username, pub_key, key, passphrase).map_err(srv_err)?;
    }

    Ok(sess)
}
