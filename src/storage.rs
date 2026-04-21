#[cfg(feature = "ssr")]
use crate::auth::{require_role, UserRole, srv_err};
use leptos::prelude::*;
use serde::{Deserialize, Serialize};

#[cfg(feature = "ssr")]
use duckdb::DuckdbConnectionManager;
#[cfg(feature = "ssr")]
use r2d2::Pool;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct VolumeEntry {
    pub id: String,
    pub name: String,
    pub size_gb: i32,
    pub usage_percent: f64,
    pub status: String, // online, offline, syncing, error
    pub last_error: Option<String>,
    pub hosts: Vec<String>,
    pub services: Vec<String>, // s3, nfs, smb
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct StorageStatus {
    pub total_gb: i64,
    pub used_gb: i64,
    pub free_gb: i64,
    pub node_count: i32,
    pub healthy_node_count: i32,
    pub diagnosis_output: String,
}

#[cfg(feature = "ssr")]
pub mod linstor {
    use super::*;
    use reqwest::Client;
    use serde_json::json;

    pub struct LinstorClient {
        base_url: String,
        http: Client,
    }

    impl LinstorClient {
        pub fn new(controller_addr: &str) -> Self {
            Self {
                base_url: format!("http://{}:3370/v1", controller_addr),
                http: Client::builder()
                    .timeout(std::time::Duration::from_secs(5))
                    .build()
                    .unwrap_or_default(),
            }
        }


        pub async fn assign_resource(&self, node_name: &str, resource_name: &str, storage_pool: &str) -> Result<(), String> {
            let body = json!([
                {
                    "resource": {
                        "node_name": node_name,
                        "props": {
                            "StorPoolName": storage_pool
                        }
                    },
                    "layer_list": ["DRBD", "STORAGE"]
                }
            ]);
            let resp = self.http.post(&format!("{}/resource-definitions/{}/resources", self.base_url, resource_name))
                .json(&body)
                .send()
                .await
                .map_err(|e| e.to_string())?;

            if !resp.status().is_success() {
                return Err(format!("Linstor Assignment Error {}: {}", resp.status(), resp.text().await.unwrap_or_default()));
            }

            Ok(())
        }

        pub async fn create_volume(
            &self, 
            pool: Pool<DuckdbConnectionManager>, 
            job_id: String, 
            name: &str, 
            size_gb: i32, 
            replicas: i32
        ) -> Result<(), String> {
            let _ = crate::jobs::update_job(pool.clone(), job_id.clone(), "running".into(), 10).await;
            let _ = crate::jobs::add_job_log(pool.clone(), job_id.clone(), format!("Linstor: Creating volume {} ({}GB, replicas={})", name, size_gb, replicas)).await;

            let body = json!({
                "resource_definition": {
                    "name": name,
                    "layer_stack": ["DRBD", "STORAGE"],
                    "auto_place": replicas
                },
                "volume_definitions": [
                    {
                        "size_kib": (size_gb as u64) * 1024 * 1024
                    }
                ]
            });

            let resp = self.http.post(&format!("{}/view/resources", self.base_url))
                .json(&body)
                .send()
                .await
                .map_err(|e| e.to_string())?;

            if !resp.status().is_success() {
                let status = resp.status();
                let err_text = resp.text().await.unwrap_or_default();
                let err_msg = format!("Linstor Error {}: {}", status, err_text);
                let _ = crate::jobs::add_job_log(pool.clone(), job_id.clone(), format!("Error: {}", err_msg)).await;
                let _ = crate::jobs::update_job(pool.clone(), job_id.clone(), "failed".into(), 0).await;
                return Err(err_msg);
            }

            let _ = crate::jobs::add_job_log(pool.clone(), job_id.clone(), "Volume created successfully.".into()).await;
            let _ = crate::jobs::update_job(pool.clone(), job_id.clone(), "completed".into(), 100).await;
            Ok(())
        }

        pub async fn delete_volume(&self, id: &str) -> Result<(), String> {
            let resp = self.http.delete(&format!("{}/resource-definitions/{}", self.base_url, id))
                .send()
                .await
                .map_err(|e| e.to_string())?;

            if !resp.status().is_success() {
                let status = resp.status();
                let err_text = resp.text().await.unwrap_or_default();
                return Err(format!("Linstor Error {}: {}", status, err_text));
            }

            Ok(())
        }

        pub async fn get_resource_placement(&self, volume_name: &str) -> Result<(String, String), String> {
            let resp = self.http.get(&format!("{}/view/resources?resource_names={}", self.base_url, volume_name))
                .send()
                .await
                .map_err(|e| e.to_string())?;

            if !resp.status().is_success() {
                return Err(format!("Linstor Placement Error: {}", resp.status()));
            }

            let resources: serde_json::Value = resp.json().await.map_err(|e| e.to_string())?;
            let arr = resources.as_array().ok_or_else(|| "Unexpected Linstor response format (expected array)".to_string())?;

            if arr.is_empty() {
                return Err(format!("Resource '{}' not found in Linstor", volume_name));
            }

            // Linstor API versions vary:
            // 1. Newer versions return an array of resource definitions, each with a "placements" list.
            // 2. Older versions (observed in debug logs) return an array of resource instances (placements) directly.
            let first = arr.first().unwrap();
            let placements = if let Some(nested) = first["placements"].as_array() {
                nested
            } else {
                // Flat format: the entire array consists of placements
                arr
            };

            if placements.is_empty() {
                return Err(format!("Resource '{}' is not assigned to any nodes", volume_name));
            }

            // Diagnostic logging for placement selection
            leptos::logging::log!("DEBUG: Found {} placements for '{}'", placements.len(), volume_name);

            // 1. Try to find a placement that is explicitly in sync
            let in_sync = placements.iter().find(|p| {
                p["state_in_sync"].as_bool().unwrap_or(false) || {
                    let state = p["state"].as_str().unwrap_or("").to_lowercase();
                    state == "uptodate" || state == "insync"
                }
            });

            // 2. Fallback to the first placement if no in-sync one is found
            let p = in_sync.or_else(|| {
                leptos::logging::log!("WARN: No UpToDate placement for '{}', falling back to first available node", volume_name);
                placements.first()
            }).unwrap();

            let node = p["node_name"].as_str().unwrap_or_default().to_string();
            let mut path = String::new();
            if let Some(vols) = p["volumes"].as_array() {
                if let Some(v) = vols.first() {
                    path = v["device_path"].as_str().unwrap_or_default().to_string();
                }
            }

            if node.is_empty() {
                return Err(format!("Selected placement for '{}' has no node name", volume_name));
            }

            Ok((node, path))
        }

        pub async fn get_error_reports(&self) -> Result<String, String> {
            let resp = self.http.get(&format!("{}/error-reports?limit=5", self.base_url))
                .send()
                .await
                .map_err(|e| e.to_string())?;
            
            if !resp.status().is_success() {
                return Err(format!("Linstor Error report fetch failed: {}", resp.status()));
            }

            let errors: serde_json::Value = resp.json().await.map_err(|e| e.to_string())?;
            Ok(serde_json::to_string_pretty(&errors).unwrap_or_default())
        }

        pub async fn get_status(&self) -> Result<StorageStatus, String> {
            let mut resp = self.http.get(&format!("{}/view/nodes", self.base_url))
                .send()
                .await
                .map_err(|e| e.to_string())?;
            
            // Fallback for older Linstor versions that don't have /view/nodes
            if resp.status() == 404 {
                resp = self.http.get(&format!("{}/nodes", self.base_url))
                    .send()
                    .await
                    .map_err(|e| e.to_string())?;
            }

            if !resp.status().is_success() {
                return Err(format!("Linstor Error {}: {}", resp.status(), resp.text().await.unwrap_or_default()));
            }

            let nodes: serde_json::Value = resp.json().await.map_err(|e| e.to_string())?;
            let mut node_count = 0;
            let mut healthy_count = 0;

            if let Some(arr) = nodes.as_array() {
                for node in arr {
                    node_count += 1;
                    if node["connection_status"].as_str().unwrap_or("") == "ONLINE" {
                        healthy_count += 1;
                    }
                }
            }

            let diag = self.get_error_reports().await.unwrap_or_else(|e| format!("Could not fetch error reports: {}", e));

            Ok(StorageStatus {
                total_gb: 0,
                used_gb: 0,
                free_gb: 0,
                node_count,
                healthy_node_count: healthy_count,
                diagnosis_output: format!("Linstor cluster is functional.\n\nRECENT ERROR REPORTS:\n{}", diag),
            })
        }

        pub async fn list_volumes(&self) -> Result<Vec<VolumeEntry>, String> {
            let resp = self.http.get(&format!("{}/view/resources", self.base_url))
                .send()
                .await
                .map_err(|e| e.to_string())?;

            if !resp.status().is_success() {
                return Err(format!("Linstor Error: {}", resp.status()));
            }

            let resources: serde_json::Value = resp.json().await.map_err(|e| e.to_string())?;
            let arr = resources.as_array().ok_or_else(|| "Expected array from Linstor".to_string())?;

            // Grouping logic for both nested and flat formats
            let mut volumes_map: std::collections::HashMap<String, Vec<serde_json::Value>> = std::collections::HashMap::new();

            for item in arr {
                if let Some(nested_placements) = item["placements"].as_array() {
                    // Nested format: Each item is a resource definition with multiple placements
                    let name = item["name"].as_str().unwrap_or_default().to_string();
                    volumes_map.entry(name).or_default().extend(nested_placements.clone());
                } else if let Some(name) = item["name"].as_str() {
                    // Flat format: Each item is a single placement instance
                    volumes_map.entry(name.to_string()).or_default().push(item.clone());
                }
            }

            let mut entries = Vec::new();
            for (name, placements) in volumes_map {
                let up_to_date = placements.iter().filter(|p| {
                    let in_sync = p["state_in_sync"].as_bool().unwrap_or(false);
                    let state = p["state"].as_str().unwrap_or("").to_lowercase();
                    let is_empty_obj = p["state"].is_object() && p["state"].as_object().map_or(false, |o| o.is_empty());
                    let has_disk_state = p["volumes"].as_array().and_then(|vols| vols.first()).and_then(|v| v["layer_data_list"].as_array()).and_then(|data| data.first()).and_then(|data| data["data"]["disk_state"].as_str()).map(|s| s == "UpToDate").unwrap_or(false);
                    
                    in_sync || state == "uptodate" || state == "insync" || is_empty_obj || has_disk_state
                }).count();
                
                let total = placements.len();
                let status = if up_to_date == total && total > 0 {
                    "healthy".to_string()
                } else if total > 0 {
                    "syncing".to_string()
                } else {
                    "unassigned".to_string()
                };

                let mut size_gb = 0;
                if let Some(p) = placements.first() {
                   if let Some(vols) = p["volumes"].as_array() {
                      if let Some(v) = vols.first() {
                         let kib = v["allocated_size_kib"].as_u64().or_else(|| v["usable_size_kib"].as_u64()).unwrap_or(0);
                         size_gb = (kib / 1024 / 1024) as i32;
                      }
                   }
                }

                entries.push(VolumeEntry {
                    id: name.clone(),
                    name,
                    size_gb,
                    usage_percent: 0.0,
                    status,
                    last_error: None,
                    hosts: placements.iter().map(|p| p["node_name"].as_str().unwrap_or("").to_string()).collect(),
                    services: vec![],
                });
            }

            entries.sort_by(|a, b| a.name.cmp(&b.name));
            Ok(entries)
        }
    }
}

// ─── Server Functions ───────────────────────────────────────────────────────

#[server(ListVolumes, "/api")]
pub async fn list_volumes() -> Result<Vec<VolumeEntry>, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        use axum::extract::Extension;
        use duckdb::DuckdbConnectionManager;
        use leptos_axum::extract;
        use r2d2::Pool;

        let Extension(pool) = extract::<Extension<Pool<DuckdbConnectionManager>>>().await
            .map_err(|e| srv_err(&format!("DB Pool failure: {:?}", e)))?;
        
        let _user = require_role(UserRole::Operator).await?;
        
        if let Some(host) = crate::hosts::get_controller_host(pool).await? {
            let client = linstor::LinstorClient::new(&host.address);
            client.list_volumes().await.map_err(srv_err)
        } else {
            Ok(vec![])
        }
    }
    #[cfg(not(feature = "ssr"))]
    { unreachable!() }
}

#[server(CreateVolume, "/api")]
pub async fn create_volume(name: String, size_gb: i32, replicas: i32) -> Result<String, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        use axum::extract::Extension;
        use duckdb::DuckdbConnectionManager;
        use leptos_axum::extract;
        use r2d2::Pool;

        let Extension(pool) = extract::<Extension<Pool<DuckdbConnectionManager>>>().await
            .map_err(|e| srv_err(&format!("DB Pool failure: {:?}", e)))?;
            
        let _user = require_role(UserRole::Operator).await?;
        
        let job_id = crate::jobs::create_job(pool.clone(), format!("Create Volume: {}", name)).await?;

        if let Some(host) = crate::hosts::get_controller_host(pool.clone()).await? {
            let client = linstor::LinstorClient::new(&host.address);
            let pool_task = pool.clone();
            let job_id_task = job_id.clone();
            
            tokio::spawn(async move {
                let _ = client.create_volume(pool_task, job_id_task, &name, size_gb, replicas).await;
            });
        } else {
            let _ = crate::jobs::add_job_log(pool.clone(), job_id.clone(), "Error: No Linstor controller found".into()).await;
            let _ = crate::jobs::update_job(pool.clone(), job_id.clone(), "failed".into(), 0).await;
        }

        Ok(job_id)
    }
    #[cfg(not(feature = "ssr"))]
    { unreachable!() }
}

#[server(DeleteVolume, "/api")]
pub async fn delete_volume(id: String) -> Result<(), ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        use axum::extract::Extension;
        use duckdb::DuckdbConnectionManager;
        use leptos_axum::extract;
        use r2d2::Pool;

        let Extension(pool) = extract::<Extension<Pool<DuckdbConnectionManager>>>().await
            .map_err(|e| srv_err(&format!("DB Pool failure: {:?}", e)))?;
            
        let _user = require_role(UserRole::Operator).await?;
        
        if let Some(host) = crate::hosts::get_controller_host(pool).await? {
            let client = linstor::LinstorClient::new(&host.address);
            client.delete_volume(&id).await.map_err(srv_err)
        } else {
            Err(srv_err("No Linstor controller found"))
        }
    }
    #[cfg(not(feature = "ssr"))]
    { unreachable!() }
}

#[server(GetStorageStatus, "/api")]
pub async fn get_storage_status() -> Result<StorageStatus, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        use axum::extract::Extension;
        use duckdb::DuckdbConnectionManager;
        use leptos_axum::extract;
        use r2d2::Pool;
        use tokio::time::{timeout, Duration};

        let Extension(pool) = extract::<Extension<Pool<DuckdbConnectionManager>>>().await
            .map_err(|e| srv_err(&format!("DB Pool failure: {:?}", e)))?;
            
        let _user = require_role(UserRole::Operator).await?;

        let res = timeout(Duration::from_secs(5), async {
            if let Some(host) = crate::hosts::get_controller_host_internal(pool.clone()).await? {
                let client = linstor::LinstorClient::new(&host.address);
                let mut status = client.get_status().await.map_err(srv_err)?;
                
                let hosts = crate::hosts::list_hosts_internal(pool.clone()).await.unwrap_or_default();
                
                let mut set = tokio::task::JoinSet::new();
                for h in hosts {
                    set.spawn_blocking(move || {
                        if let Ok(sess) = crate::hosts::establish_ssh_session(&h) {
                            if let Ok((0, out, _)) = crate::ssh::run_remote_script(&sess, "zpool list -Hp -o size cloudlab_zpool", h.password.as_deref()) {
                                if let Ok(bytes) = out.trim().parse::<i64>() {
                                    return bytes / (1024 * 1024 * 1024);
                                }
                            }
                        }
                        h.zfs_pool_size_gb as i64
                    });
                }
                
                let mut sum_gb = 0;
                while let Some(res) = set.join_next().await {
                    if let Ok(cap) = res {
                        sum_gb += cap;
                    }
                }
                status.total_gb = sum_gb;
                
                let vols = client.list_volumes().await.unwrap_or_default();
                status.used_gb = vols.iter().map(|v| v.size_gb as i64).sum();
                status.free_gb = status.total_gb - status.used_gb;
                
                Ok(status)
            } else {
                Ok(StorageStatus {
                    total_gb: 0, used_gb: 0, free_gb: 0,
                    node_count: 0, healthy_node_count: 0,
                    diagnosis_output: "No controller found in database. Please add a host and run setup.".into(),
                })
            }
        }).await;

        match res {
            Ok(inner) => inner,
            Err(_) => Err(srv_err("Linstor get_storage_status timed out"))
        }
    }
    #[cfg(not(feature = "ssr"))]
    { unreachable!() }
}

#[server(InitializeVolume, "/api")]
pub async fn initialize_volume(volume_id: String) -> Result<(), ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        use axum::extract::Extension;
        use duckdb::DuckdbConnectionManager;
        use leptos_axum::extract;
        use r2d2::Pool;
        use tokio::time::{sleep, Duration};

        let Extension(pool) = extract::<Extension<Pool<DuckdbConnectionManager>>>().await
            .map_err(|e| srv_err(&format!("DB Pool failure: {:?}", e)))?;
            
        let _user = require_role(UserRole::Operator).await?;

        // 1. Try to find existing placement
        let initial_find = crate::storage_ssr::srv::find_active_host_for_volume(pool.clone(), &volume_id).await;

        let (host, device_path) = match initial_find {
            Ok(found) => found,
            Err(e) if e.to_string().contains("is not assigned to any nodes") => {
                // AUTO-ASSIGNMENT LOGIC
                let controller = crate::hosts::get_controller_host(pool.clone()).await?
                    .ok_or_else(|| srv_err("No Linstor controller found for auto-assignment"))?;
                let client = linstor::LinstorClient::new(&controller.address);

                // Use the controller host itself as the primary target for core volumes
                leptos::logging::log!("Auto-assigning resource '{}' to node '{}'", volume_id, controller.name);
                client.assign_resource(&controller.name, &volume_id, "cloudlab_pool").await.map_err(srv_err)?;

                // Polling wait for the placement to become available
                let mut retry_count = 0;
                loop {
                    sleep(Duration::from_secs(2)).await;
                    match crate::storage_ssr::srv::find_active_host_for_volume(pool.clone(), &volume_id).await {
                        Ok(found) => break found,
                        Err(e) => {
                            retry_count += 1;
                            if retry_count > 30 {
                                return Err(srv_err(&format!("Timed out waiting for Linstor placement after 60s: {}", e)));
                            }
                            leptos::logging::log!("Waiting for placement '{}' (retry {}/30)...", volume_id, retry_count);
                        }
                    }
                }
            },
            Err(e) => return Err(e),
        };

        let mount_point = match volume_id.as_str() {
            "cloudlab-isos" => "/mnt/isos",
            "llm-models" => "/mnt/llm-models",
            _ => return Err(srv_err("Unknown volume ID for initialization")),
        };

        crate::storage_ssr::srv::initialize_volume(&host, &volume_id, &device_path, mount_point)
            .await.map_err(srv_err)
    }
    #[cfg(not(feature = "ssr"))]
    { unreachable!() }
}

#[server(ListFiles, "/api")]
pub async fn list_files(volume_id: String, path: String) -> Result<Vec<crate::storage_ssr::FileEntry>, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        use axum::extract::Extension;
        use duckdb::DuckdbConnectionManager;
        use leptos_axum::extract;
        use r2d2::Pool;

        let Extension(pool) = extract::<Extension<Pool<DuckdbConnectionManager>>>().await
            .map_err(|e| srv_err(&format!("DB Pool failure: {:?}", e)))?;
            
        let _user = require_role(UserRole::Operator).await?;

        let (host, _) = crate::storage_ssr::srv::find_active_host_for_volume(pool, &volume_id).await?;
        let root = match volume_id.as_str() {
            "cloudlab-isos" => "/mnt/isos",
            "llm-models" => "/mnt/llm-models",
            _ => return Err(srv_err("Unknown volume ID")),
        };
        let full_path = format!("{}/{}", root, path.trim_start_matches('/'));

        crate::storage_ssr::srv::list_files(&host, &full_path).await.map_err(srv_err)
    }
    #[cfg(not(feature = "ssr"))]
    { unreachable!() }
}

#[server(CreateFolder, "/api")]
pub async fn create_folder(volume_id: String, sub_path: String) -> Result<(), ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        use axum::extract::Extension;
        use duckdb::DuckdbConnectionManager;
        use leptos_axum::extract;
        use r2d2::Pool;

        let Extension(pool) = extract::<Extension<Pool<DuckdbConnectionManager>>>().await
            .map_err(|e| srv_err(&format!("DB Pool failure: {:?}", e)))?;
            
        let _user = require_role(UserRole::Operator).await?;

        let (host, _) = crate::storage_ssr::srv::find_active_host_for_volume(pool, &volume_id).await?;
        let root = match volume_id.as_str() {
            "cloudlab-isos" => "/mnt/isos",
            "llm-models" => "/mnt/llm-models",
            _ => return Err(srv_err("Unknown volume ID")),
        };
        let full_path = format!("{}/{}", root, sub_path.trim_start_matches('/'));

        crate::storage_ssr::srv::create_folder(&host, &full_path).await.map_err(srv_err)
    }
    #[cfg(not(feature = "ssr"))]
    { unreachable!() }
}

#[server(RenameFile, "/api")]
pub async fn rename_file(volume_id: String, old_path: String, new_path: String) -> Result<(), ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        use axum::extract::Extension;
        use duckdb::DuckdbConnectionManager;
        use leptos_axum::extract;
        use r2d2::Pool;

        let Extension(pool) = extract::<Extension<Pool<DuckdbConnectionManager>>>().await
            .map_err(|e| srv_err(&format!("DB Pool failure: {:?}", e)))?;
            
        let _user = require_role(UserRole::Operator).await?;

        let (host, _) = crate::storage_ssr::srv::find_active_host_for_volume(pool, &volume_id).await?;
        let root = match volume_id.as_str() {
            "cloudlab-isos" => "/mnt/isos",
            "llm-models" => "/mnt/llm-models",
            _ => return Err(srv_err("Unknown volume ID")),
        };
        let full_old = format!("{}/{}", root, old_path.trim_start_matches('/'));
        let full_new = format!("{}/{}", root, new_path.trim_start_matches('/'));

        crate::storage_ssr::srv::rename_path(&host, &full_old, &full_new).await.map_err(srv_err)
    }
    #[cfg(not(feature = "ssr"))]
    { unreachable!() }
}

#[server(DeleteFiles, "/api")]
pub async fn delete_files(volume_id: String, sub_paths: Vec<String>) -> Result<(), ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        use axum::extract::Extension;
        use duckdb::DuckdbConnectionManager;
        use leptos_axum::extract;
        use r2d2::Pool;

        let Extension(pool) = extract::<Extension<Pool<DuckdbConnectionManager>>>().await
            .map_err(|e| srv_err(&format!("DB Pool failure: {:?}", e)))?;
            
        let _user = require_role(UserRole::Operator).await?;

        let (host, _) = crate::storage_ssr::srv::find_active_host_for_volume(pool, &volume_id).await?;
        let root = match volume_id.as_str() {
            "cloudlab-isos" => "/mnt/isos",
            "llm-models" => "/mnt/llm-models",
            _ => return Err(srv_err("Unknown volume ID")),
        };
        
        let full_paths: Vec<String> = sub_paths.into_iter()
            .map(|p| format!("{}/{}", root, p.trim_start_matches('/')))
            .collect();

        crate::storage_ssr::srv::delete_paths(&host, full_paths).await.map_err(srv_err)
    }
    #[cfg(not(feature = "ssr"))]
    { unreachable!() }
}

#[server(GetFilePreview, "/api")]
pub async fn get_file_preview(volume_id: String, path: String) -> Result<String, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        use axum::extract::Extension;
        use duckdb::DuckdbConnectionManager;
        use leptos_axum::extract;
        use r2d2::Pool;

        let Extension(pool) = extract::<Extension<Pool<DuckdbConnectionManager>>>().await
            .map_err(|e| srv_err(&format!("DB Pool failure: {:?}", e)))?;
            
        let _user = require_role(UserRole::Operator).await?;

        let (host, _) = crate::storage_ssr::srv::find_active_host_for_volume(pool, &volume_id).await?;
        let root = match volume_id.as_str() {
            "cloudlab-isos" => "/mnt/isos",
            "llm-models" => "/mnt/llm-models",
            _ => return Err(srv_err("Unknown volume ID")),
        };
        let full_path = format!("{}/{}", root, path.trim_start_matches('/'));

        crate::storage_ssr::srv::read_file_preview(&host, &full_path, 1024).await.map_err(srv_err)
    }
    #[cfg(not(feature = "ssr"))]
    { unreachable!() }
}
