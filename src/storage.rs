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
                    .timeout(std::time::Duration::from_secs(60))
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
            let _ = crate::jobs::add_job_log(pool.clone(), job_id.clone(), format!("Linstor: Creating resource definition {}", name)).await;

            let rd_body = json!({
                "resource_definition": {
                    "name": name,
                    "resource_group_name": "DfltRscGrp"
                }
            });

            let rd_resp = self.http.post(&format!("{}/resource-definitions", self.base_url))
                .json(&rd_body)
                .send()
                .await
                .map_err(|e| e.to_string())?;

            if !rd_resp.status().is_success() {
                let status = rd_resp.status();
                let err_text = rd_resp.text().await.unwrap_or_default();
                if !err_text.contains("already exists") {
                    let err_msg = format!("Linstor RD Error {}: {}", status, err_text);
                    let _ = crate::jobs::add_job_log(pool.clone(), job_id.clone(), format!("Error: {}", err_msg)).await;
                    let _ = crate::jobs::update_job(pool.clone(), job_id.clone(), "failed".into(), 0).await;
                    return Err(err_msg);
                }
            }

            let _ = crate::jobs::add_job_log(pool.clone(), job_id.clone(), format!("Linstor: Checking volume definitions for {}", name)).await;
            
            let list_vd_resp = self.http.get(&format!("{}/resource-definitions/{}/volume-definitions", self.base_url, name))
                .send()
                .await
                .map_err(|e| e.to_string())?;

            let mut volumes_exist = false;
            if list_vd_resp.status().is_success() {
                let existing_vds: serde_json::Value = list_vd_resp.json().await.unwrap_or_default();
                if let Some(arr) = existing_vds.as_array() {
                    if !arr.is_empty() {
                        volumes_exist = true;
                        let _ = crate::jobs::add_job_log(pool.clone(), job_id.clone(), "Linstor: Volume definition already exists, skipping creation.".into()).await;
                    }
                }
            }

            if !volumes_exist {
                let _ = crate::jobs::add_job_log(pool.clone(), job_id.clone(), format!("Linstor: Creating volume definition for {} ({}GB)", name, size_gb)).await;

                let vd_body = json!({
                    "volume_definition": {
                        "size_kib": (size_gb as u64) * 1024 * 1024
                    }
                });

                let vd_resp = self.http.post(&format!("{}/resource-definitions/{}/volume-definitions", self.base_url, name))
                    .json(&vd_body)
                    .send()
                    .await
                    .map_err(|e| e.to_string())?;

                if !vd_resp.status().is_success() {
                    let status = vd_resp.status();
                    let err_text = vd_resp.text().await.unwrap_or_default();
                    if !err_text.contains("already exists") {
                        let err_msg = format!("Linstor VD Error {}: {}", status, err_text);
                        let _ = crate::jobs::add_job_log(pool.clone(), job_id.clone(), format!("Error: {}", err_msg)).await;
                        let _ = crate::jobs::update_job(pool.clone(), job_id.clone(), "failed".into(), 0).await;
                        return Err(err_msg);
                    }
                }
            } else {
                // If it exists, try to resize it to the requested size just in case it was created smaller
                let _ = self.resize_volume(name, 0, size_gb).await;
            }

            let _ = crate::jobs::add_job_log(pool.clone(), job_id.clone(), format!("Linstor: Auto-placing {} with {} replicas", name, replicas)).await;

            let ap_body = json!({
                "select_filter": {
                    "place_count": replicas
                }
            });

            let ap_resp = self.http.post(&format!("{}/resource-definitions/{}/autoplace", self.base_url, name))
                .json(&ap_body)
                .send()
                .await
                .map_err(|e| e.to_string())?;

            if !ap_resp.status().is_success() {
                let status = ap_resp.status();
                let err_text = ap_resp.text().await.unwrap_or_default();
                let err_msg = format!("Linstor Autoplace Error {}: {}", status, err_text);
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

        pub async fn resize_volume(&self, resource_name: &str, volume_nr: i32, new_size_gb: i32) -> Result<(), String> {
            let body = json!({
                "volume_definition": {
                    "size_kib": (new_size_gb as u64) * 1024 * 1024
                }
            });
            let resp = self.http.put(&format!("{}/resource-definitions/{}/volume-definitions/{}", self.base_url, resource_name, volume_nr))
                .json(&body)
                .send()
                .await
                .map_err(|e| e.to_string())?;

            if !resp.status().is_success() {
                return Err(format!("Linstor Resize Error {}: {}", resp.status(), resp.text().await.unwrap_or_default()));
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

            let first = arr.first().unwrap();
            let placements = if let Some(nested) = first["placements"].as_array() {
                nested
            } else {
                arr
            };

            if placements.is_empty() {
                return Err(format!("Resource '{}' is not assigned to any nodes", volume_name));
            }

            // 1. Try to find a placement that is explicitly in sync
            let in_sync = placements.iter().find(|p| {
                p["state_in_sync"].as_bool().unwrap_or(false) || {
                    let state = p["state"].as_str().unwrap_or("").to_lowercase();
                    state == "uptodate" || state == "insync"
                }
            });

            let p = in_sync.or_else(|| {
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

        pub async fn wait_for_resource_uptodate(
            &self, 
            volume_name: &str, 
            target_node: &str,
            pool: Option<Pool<DuckdbConnectionManager>>,
            job_id: Option<String>
        ) -> Result<(), String> {
            let mut attempts = 0;
            let max_attempts = 60; // Increased to 60s
            leptos::logging::log!("DEBUG: Waiting for resource '{}' to be UpToDate on node '{}'", volume_name, target_node);
            
            while attempts < max_attempts {
                let resp = self.http.get(&format!("{}/view/resources?resource_names={}", self.base_url, volume_name))
                    .send()
                    .await
                    .map_err(|e| e.to_string())?;
                
                if resp.status().is_success() {
                    if let Ok(resources) = resp.json::<serde_json::Value>().await {
                        let arr = resources.as_array().map(|a| a.as_slice()).unwrap_or(&[]);
                        let placements = if let Some(first) = arr.first() {
                            if let Some(nested) = first["placements"].as_array() { nested.as_slice() } else { arr }
                        } else { arr };

                        let mut found_node = false;
                        for p in placements {
                            let node_name = p["node_name"].as_str().unwrap_or_default();
                            if node_name == target_node {
                                found_node = true;
                                let state = if let Some(s) = p["state"].as_str() {
                                    s.to_string()
                                } else if let Some(obj) = p["state"].as_object() {
                                    if let Some(ds) = obj.get("disk_state").and_then(|v| v.as_str()) {
                                        ds.to_string()
                                    } else if let Some(vols) = p["volumes"].as_array() {
                                        // Aggregate state from all volumes
                                        let all_uptodate = !vols.is_empty() && vols.iter().all(|v| {
                                            v["state"]["disk_state"].as_str().map(|s| s.to_lowercase()) == Some("uptodate".into())
                                        });
                                        if all_uptodate { "UpToDate".into() } else { "Syncing".into() }
                                    } else {
                                        "unknown".to_string()
                                    }
                                } else {
                                    "unknown".to_string()
                                };
                                let in_sync = p["state_in_sync"].as_bool().unwrap_or_else(|| {
                                    state.to_lowercase() == "uptodate" || state.to_lowercase() == "insync"
                                });
                                
                                let msg = format!("Storage Status [{}]: node={}, state={}, in_sync={}", volume_name, target_node, state, in_sync);
                                leptos::logging::log!("DEBUG: {}", msg);
                                
                                if let (Some(p), Some(jid)) = (&pool, &job_id) {
                                    let _ = crate::jobs::add_job_log(p.clone(), jid.clone(), msg).await;
                                }

                                if state.to_lowercase() == "uptodate" || in_sync {
                                    return Ok(());
                                }
                            }
                        }
                        if !found_node {
                            let available_nodes: Vec<&str> = placements.iter().filter_map(|p| p["node_name"].as_str()).collect();
                            let msg = format!("WARN: Node '{}' not found in placements for '{}'. Available: {:?}", target_node, volume_name, available_nodes);
                            leptos::logging::log!("{}", msg);
                            if let (Some(p), Some(jid)) = (&pool, &job_id) {
                                let _ = crate::jobs::add_job_log(p.clone(), jid.clone(), msg).await;
                            }
                        }
                    }
                } else {
                    let err_msg = format!("ERROR: Linstor API failed with status {}: {}", resp.status(), resp.text().await.unwrap_or_default());
                    leptos::logging::log!("{}", err_msg);
                    if let (Some(p), Some(jid)) = (&pool, &job_id) {
                        let _ = crate::jobs::add_job_log(p.clone(), jid.clone(), err_msg).await;
                    }
                }
                attempts += 1;
                tokio::time::sleep(std::time::Duration::from_secs(1)).await;
            }
            let raw_status = match self.http.get(&format!("{}/view/resources/{}", self.base_url, volume_name)).send().await {
                Ok(resp) => resp.text().await.unwrap_or_else(|_| "Could not read status body".into()),
                Err(e) => format!("Could not fetch status: {}", e),
            };
            let err_reports = self.get_error_reports().await.unwrap_or_else(|e| format!("Could not fetch error reports: {} ", e));
            Err(format!("Timeout waiting for resource {} to be UpToDate on {}.\n\nRAW LINSTOR STATUS:\n{}\n\nLINSTOR ERROR REPORTS:\n{}", volume_name, target_node, raw_status, err_reports))
        }

        pub async fn get_error_reports(&self) -> Result<String, String> {
            let resp = self.http.get(&format!("{}/error-reports?limit=50", self.base_url))
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
