use serde::{Serialize, Deserialize};
use leptos::prelude::*;
#[cfg(feature = "ssr")]
use duckdb::OptionalExt;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LLM {
    pub id: String,
    pub host_id: String,
    pub host_name: String,
    pub name: String,
    pub provider: String,
    pub model_name: String,
    pub status: String,
    pub repo_id: Option<String>,
    pub size_bytes: i64,
    pub download_status: String,
    pub last_synced_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatThread {
    pub id: String,
    pub llm_id: String,
    pub title: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct HFModel {
    pub id: String,
    pub downloads: Option<i64>,
    pub likes: Option<i64>,
}

pub fn srv_err(msg: impl std::fmt::Display) -> ServerFnError {
    ServerFnError::new(msg.to_string())
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub role: String,
    pub content: String,
}

#[server(ListLLMs, "/api")]
pub async fn list_llms() -> Result<Vec<LLM>, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        use axum::extract::Extension;
        use duckdb::DuckdbConnectionManager;
        use leptos_axum::extract;
        use r2d2::Pool;

        let Extension(pool) = extract::<Extension<Pool<DuckdbConnectionManager>>>()
            .await
            .map_err(|_| srv_err("Failed to extract DB pool"))?;

        let llms = tokio::task::spawn_blocking(move || -> Result<Vec<LLM>, ServerFnError> {
            let conn = pool.get().map_err(srv_err)?;
            let mut stmt = conn
                .prepare(
                    "SELECT l.id, l.host_id, h.name as host_name, l.name, l.provider, l.model_name, l.status, l.repo_id, l.size_bytes, l.download_status, CAST(l.last_synced_at AS VARCHAR) \
                     FROM llms l \
                     LEFT JOIN hosts h ON l.host_id = h.id \
                     ORDER BY l.name ASC;",
                )
                .map_err(srv_err)?;
            let mut rows = stmt.query(duckdb::params![]).map_err(srv_err)?;
            let mut llms = Vec::new();
            while let Some(row) = rows.next().map_err(srv_err)? {
                llms.push(LLM {
                    id: row.get::<_, String>(0).map_err(srv_err)?,
                    host_id: row.get::<_, String>(1).map_err(srv_err).unwrap_or_default(),
                    host_name: row.get::<_, String>(2).map_err(srv_err).unwrap_or_else(|_| "Unknown".into()),
                    name: row.get::<_, String>(3).map_err(srv_err)?,
                    provider: row.get::<_, String>(4).map_err(srv_err)?,
                    model_name: row.get::<_, String>(5).map_err(srv_err)?,
                    status: row.get::<_, String>(6).map_err(srv_err)?,
                    repo_id: row.get::<_, Option<String>>(7).map_err(srv_err)?,
                    size_bytes: row.get::<_, i64>(8).map_err(srv_err).unwrap_or(0),
                    download_status: row.get::<_, String>(9).map_err(srv_err).unwrap_or_else(|_| "none".into()),
                    last_synced_at: row.get::<_, Option<String>>(10).map_err(srv_err)?,
                });
            }
            Ok(llms)
        })
        .await
        .map_err(srv_err)??;

        Ok(llms)
    }
    #[cfg(not(feature = "ssr"))]
    { unreachable!() }
}

#[server(ToggleLLMStatus, "/api")]
pub async fn toggle_llm_status(id: String) -> Result<String, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        use axum::extract::Extension;
        use duckdb::DuckdbConnectionManager;
        use leptos_axum::extract;
        use r2d2::Pool;
        use duckdb::params;

        let Extension(pool) = extract::<Extension<Pool<DuckdbConnectionManager>>>()
            .await
            .map_err(|_| srv_err("Failed to extract DB pool"))?;

        let new_status = tokio::task::spawn_blocking(move || -> Result<String, ServerFnError> {
            let conn = pool.get().map_err(srv_err)?;
            let mut stmt = conn.prepare("SELECT status FROM llms WHERE id=?;").map_err(srv_err)?;
            let current_status: String = stmt.query_row(params![id], |row| row.get(0)).map_err(srv_err)?;
            
            let next_status = if current_status == "online" { "offline" } else { "online" };
            
            conn.execute("UPDATE llms SET status=? WHERE id=?;", params![next_status, id]).map_err(srv_err)?;
            
            Ok(next_status.to_string())
        })
        .await
        .map_err(srv_err)??;

        Ok(new_status)
    }
    #[cfg(not(feature = "ssr"))]
    { unreachable!() }
}

#[server(CreateLLM, "/api")]
pub async fn create_llm(
    host_id: String,
    name: String,
    provider: String,
    model_name: String,
) -> Result<(), ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        use axum::extract::Extension;
        use duckdb::DuckdbConnectionManager;
        use leptos_axum::extract;
        use r2d2::Pool;
        use duckdb::params;

        let Extension(pool) = extract::<Extension<Pool<DuckdbConnectionManager>>>()
            .await
            .map_err(|_| srv_err("Failed to extract DB pool"))?;

        tokio::task::spawn_blocking(move || -> Result<(), ServerFnError> {
            let conn = pool.get().map_err(srv_err)?;
            let id = uuid::Uuid::new_v4().to_string();
            conn.execute(
                "INSERT INTO llms (id, host_id, name, provider, model_name, status) VALUES (?, ?, ?, ?, ?, 'offline');",
                params![id, host_id, name, provider, model_name],
            ).map_err(srv_err)?;
            Ok(())
        })
        .await
        .map_err(srv_err)??;

        Ok(())
    }
    #[cfg(not(feature = "ssr"))]
    { unreachable!() }
}

#[server(UpdateLLM, "/api")]
pub async fn update_llm(
    id: String,
    host_id: String,
    name: String,
    provider: String,
    model_name: String,
) -> Result<(), ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        use axum::extract::Extension;
        use duckdb::DuckdbConnectionManager;
        use leptos_axum::extract;
        use r2d2::Pool;
        use duckdb::params;

        let Extension(pool) = extract::<Extension<Pool<DuckdbConnectionManager>>>()
            .await
            .map_err(|_| srv_err("Failed to extract DB pool"))?;

        tokio::task::spawn_blocking(move || -> Result<(), ServerFnError> {
            let conn = pool.get().map_err(srv_err)?;
            conn.execute(
                "UPDATE llms SET host_id=?, name=?, provider=?, model_name=? WHERE id=?;",
                params![host_id, name, provider, model_name, id],
            ).map_err(srv_err)?;
            Ok(())
        })
        .await
        .map_err(srv_err)??;

        Ok(())
    }
    #[cfg(not(feature = "ssr"))]
    { unreachable!() }
}

#[server(DeleteLLM, "/api")]
pub async fn delete_llm(id: String) -> Result<(), ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        use axum::extract::Extension;
        use duckdb::DuckdbConnectionManager;
        use leptos_axum::extract;
        use r2d2::Pool;
        use duckdb::params;

        let Extension(pool) = extract::<Extension<Pool<DuckdbConnectionManager>>>()
            .await
            .map_err(|_| srv_err("Failed to extract DB pool"))?;

        tokio::task::spawn_blocking(move || -> Result<(), ServerFnError> {
            let conn = pool.get().map_err(srv_err)?;
            conn.execute("DELETE FROM llms WHERE id=?;", params![id]).map_err(srv_err)?;
            Ok(())
        })
        .await
        .map_err(srv_err)??;

        Ok(())
    }
    #[cfg(not(feature = "ssr"))]
    { unreachable!() }
}

// ─── Chat ────────────────────────────────────────────────────────────────────

#[server(SendChatMessage, "/api")]
pub async fn send_chat_message(
    llm_id: String,
    thread_id: Option<String>,
    messages: Vec<ChatMessage>,
) -> Result<(String, String), ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        use axum::extract::Extension;
        use duckdb::DuckdbConnectionManager;
        use leptos_axum::extract;
        use r2d2::Pool;

        let Extension(pool) = extract::<Extension<Pool<DuckdbConnectionManager>>>()
            .await
            .map_err(|_| srv_err("Failed to extract DB pool"))?;

        // 1. Resolve host and model
        let pool_for_resolve = pool.clone();
        let llm_id_for_resolve = llm_id.clone();
        let (host_addr, model_name) = tokio::task::spawn_blocking(move || -> Result<(String, String), ServerFnError> {
            let conn = pool_for_resolve.get().map_err(srv_err)?;
            let mut stmt = conn.prepare("SELECT h.address, l.model_name FROM llms l JOIN hosts h ON l.host_id = h.id WHERE l.id = ?;").map_err(srv_err)?;
            let res = stmt.query_row(duckdb::params![llm_id_for_resolve], |row| Ok((row.get(0)?, row.get(1)?))).map_err(srv_err)?;
            Ok(res)
        }).await.map_err(srv_err)??;

        // 2. Persist user message if thread exists
        if let (Some(tid), Some(last_msg)) = (&thread_id, messages.last()) {
            if last_msg.role == "user" {
                let pool_c = pool.clone();
                let tid_c = tid.clone();
                let msg_c = last_msg.content.clone();
                let _ = tokio::task::spawn_blocking(move || {
                    if let Ok(conn) = pool_c.get() {
                        let id = uuid::Uuid::new_v4().to_string();
                        let _ = conn.execute("INSERT INTO chat_messages (id, thread_id, role, content) VALUES (?, ?, 'user', ?);", duckdb::params![id, tid_c, msg_c]);
                        let _ = conn.execute("UPDATE chat_threads SET updated_at = CURRENT_TIMESTAMP WHERE id = ?;", duckdb::params![tid_c]);
                    }
                }).await;
            }
        }

        // 3. Call Fox API with retry logic
        let url = format!("http://{}:8080/v1/chat/completions", host_addr);
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(120))
            .build()
            .map_err(|e| srv_err(&format!("Failed to build client: {}", e)))?;
        
        let mut final_content = String::new();
        let mut retry_count = 0;
        let mut last_json = serde_json::Value::Null;
        
        while retry_count < 5 {
            let resp = client.post(&url)
                .json(&serde_json::json!({
                    "model": model_name,
                    "messages": messages,
                    "stream": false,
                    "temperature": 0.7,
                    "top_p": 0.9,
                    "max_tokens": 2048
                }))
                .send()
                .await
                .map_err(|e| srv_err(&format!("Failed to connect to Fox: {}", e)))?;

            if !resp.status().is_success() {
                let body = resp.text().await.unwrap_or_default();
                return Err(srv_err(&format!("Fox Chat API Error: {}", body)));
            }

            let json: serde_json::Value = resp.json().await.map_err(|e| srv_err(&format!("Failed to parse Fox response: {}", e)))?;
            last_json = json.clone();
            
            let choice = &json["choices"][0];
            let content = choice["message"]["content"].as_str().unwrap_or("").to_string();
            let finish_reason = choice["finish_reason"].as_str().unwrap_or("unknown");
            
            if !content.is_empty() {
                final_content = content;
                break;
            } else if finish_reason == "content_filter" {
                return Err(srv_err("Fox Engine blocked response due to content filters."));
            }
            
            retry_count += 1;
            if retry_count < 5 {
                tokio::time::sleep(std::time::Duration::from_millis(1000)).await;
            }
        }

        if final_content.is_empty() {
            let usage = &last_json["usage"];
            let prompt_tokens = usage["prompt_tokens"].as_u64().unwrap_or(0);
            let completion_tokens = usage["completion_tokens"].as_u64().unwrap_or(0);
            return Err(srv_err(&format!(
                "Model generation failure ({} retries). Finish reason: {}. Usage: {} in, {} out. Json: {:?}", 
                retry_count, 
                last_json["choices"][0]["finish_reason"].as_str().unwrap_or("unknown"),
                prompt_tokens,
                completion_tokens,
                last_json
            )));
        }

        // 4. Create thread if not provided, and persist response
        let mut actual_thread_id = thread_id.unwrap_or_default();
        if actual_thread_id.is_empty() {
            let title = messages.first().map(|m| {
                let s = m.content.chars().take(30).collect::<String>();
                if m.content.len() > 30 { format!("{}...", s) } else { s }
            }).unwrap_or_else(|| "New Chat".to_string());

            let nt = create_chat_thread(llm_id, title).await?;
            actual_thread_id = nt.id;
        }

        let pool_c = pool.clone();
        let tid_c = actual_thread_id.clone();
        let content_c = final_content.clone();
        let _ = tokio::task::spawn_blocking(move || {
            if let Ok(conn) = pool_c.get() {
                let id = uuid::Uuid::new_v4().to_string();
                let _ = conn.execute("INSERT INTO chat_messages (id, thread_id, role, content) VALUES (?, ?, 'assistant', ?);", duckdb::params![id, tid_c, content_c]);
                let _ = conn.execute("UPDATE chat_threads SET updated_at = CURRENT_TIMESTAMP WHERE id = ?;", duckdb::params![tid_c]);
            }
        }).await;

        Ok((actual_thread_id, final_content))
    }
    #[cfg(not(feature = "ssr"))]
    { unreachable!() }
}

#[server(ListChatThreads, "/api")]
pub async fn list_chat_threads(llm_id: String) -> Result<Vec<ChatThread>, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        use axum::extract::Extension;
        use duckdb::DuckdbConnectionManager;
        use leptos_axum::extract;
        use r2d2::Pool;

        let Extension(pool) = extract::<Extension<Pool<DuckdbConnectionManager>>>()
            .await
            .map_err(|_| srv_err("Failed to extract DB pool"))?;

        tokio::task::spawn_blocking(move || -> Result<Vec<ChatThread>, ServerFnError> {
            let conn = pool.get().map_err(srv_err)?;
            let mut stmt = conn.prepare("SELECT id, llm_id, title, CAST(created_at AS VARCHAR), CAST(updated_at AS VARCHAR) FROM chat_threads WHERE llm_id = ? ORDER BY updated_at DESC;").map_err(srv_err)?;
            let rows = stmt.query_map(duckdb::params![llm_id], |row| {
                Ok(ChatThread {
                    id: row.get(0)?,
                    llm_id: row.get(1)?,
                    title: row.get(2)?,
                    created_at: row.get(3)?,
                    updated_at: row.get(4)?,
                })
            }).map_err(srv_err)?;

            let mut threads = Vec::new();
            for r in rows { threads.push(r.map_err(srv_err)?); }
            Ok(threads)
        }).await.map_err(srv_err)?
    }
    #[cfg(not(feature = "ssr"))]
    { unreachable!() }
}

#[server(GetChatHistory, "/api")]
pub async fn get_chat_history(thread_id: String) -> Result<Vec<ChatMessage>, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        use axum::extract::Extension;
        use duckdb::DuckdbConnectionManager;
        use leptos_axum::extract;
        use r2d2::Pool;

        let Extension(pool) = extract::<Extension<Pool<DuckdbConnectionManager>>>()
            .await
            .map_err(|_| srv_err("Failed to extract DB pool"))?;

        tokio::task::spawn_blocking(move || -> Result<Vec<ChatMessage>, ServerFnError> {
            let conn = pool.get().map_err(srv_err)?;
            let mut stmt = conn.prepare("SELECT role, content FROM chat_messages WHERE thread_id = ? ORDER BY created_at ASC;").map_err(srv_err)?;
            let rows = stmt.query_map(duckdb::params![thread_id], |row| {
                Ok(ChatMessage {
                    role: row.get(0)?,
                    content: row.get(1)?,
                })
            }).map_err(srv_err)?;

            let mut messages = Vec::new();
            for r in rows { messages.push(r.map_err(srv_err)?); }
            Ok(messages)
        }).await.map_err(srv_err)?
    }
    #[cfg(not(feature = "ssr"))]
    { unreachable!() }
}

#[server(CreateChatThread, "/api")]
pub async fn create_chat_thread(llm_id: String, title: String) -> Result<ChatThread, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        use axum::extract::Extension;
        use duckdb::DuckdbConnectionManager;
        use leptos_axum::extract;
        use r2d2::Pool;

        let Extension(pool) = extract::<Extension<Pool<DuckdbConnectionManager>>>()
            .await
            .map_err(|_| srv_err("Failed to extract DB pool"))?;

        tokio::task::spawn_blocking(move || -> Result<ChatThread, ServerFnError> {
            let conn = pool.get().map_err(srv_err)?;
            let id = uuid::Uuid::new_v4().to_string();
            conn.execute("INSERT INTO chat_threads (id, llm_id, title) VALUES (?, ?, ?);", duckdb::params![id, llm_id, title]).map_err(srv_err)?;
            
            let mut stmt = conn.prepare("SELECT id, llm_id, title, CAST(created_at AS VARCHAR), CAST(updated_at AS VARCHAR) FROM chat_threads WHERE id = ?;").map_err(srv_err)?;
            let thread = stmt.query_row(duckdb::params![id], |row| {
                Ok(ChatThread {
                    id: row.get(0)?,
                    llm_id: row.get(1)?,
                    title: row.get(2)?,
                    created_at: row.get(3)?,
                    updated_at: row.get(4)?,
                })
            }).map_err(srv_err)?;
            Ok(thread)
        }).await.map_err(srv_err)?
    }
    #[cfg(not(feature = "ssr"))]
    { unreachable!() }
}

#[server(DeleteChatThread, "/api")]
pub async fn delete_chat_thread(thread_id: String) -> Result<(), ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        use axum::extract::Extension;
        use duckdb::DuckdbConnectionManager;
        use leptos_axum::extract;
        use r2d2::Pool;

        let Extension(pool) = extract::<Extension<Pool<DuckdbConnectionManager>>>()
            .await
            .map_err(|_| srv_err("Failed to extract DB pool"))?;

        tokio::task::spawn_blocking(move || -> Result<(), ServerFnError> {
            let conn = pool.get().map_err(srv_err)?;
            conn.execute("DELETE FROM chat_messages WHERE thread_id = ?;", duckdb::params![thread_id]).map_err(srv_err)?;
            conn.execute("DELETE FROM chat_threads WHERE id = ?;", duckdb::params![thread_id]).map_err(srv_err)?;
            Ok(())
        }).await.map_err(srv_err)?
    }
    #[cfg(not(feature = "ssr"))]
    { unreachable!() }
}

// ─── Fox Operations ──────────────────────────────────────────────────────────

#[server(SyncHostModels, "/api")]
pub async fn sync_host_models(host_id: String) -> Result<(), ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        use axum::extract::Extension;
        use duckdb::DuckdbConnectionManager;
        use leptos_axum::extract;
        use r2d2::Pool;

        let Extension(pool) = extract::<Extension<Pool<DuckdbConnectionManager>>>()
            .await
            .map_err(|_| srv_err("Failed to extract DB pool"))?;

        // 1. Get host address
        let pool_clone = pool.clone();
        let host_id_clone = host_id.clone();
        let host_addr = tokio::task::spawn_blocking(move || -> Result<String, ServerFnError> {
            let conn = pool_clone.get().map_err(srv_err)?;
            let res: Result<Option<String>, duckdb::Error> = conn.query_row("SELECT address FROM hosts WHERE id = ?;", duckdb::params![host_id_clone], |r| r.get(0)).optional();
            let addr = res.map_err(srv_err)?;
            addr.ok_or_else(|| srv_err("Host not found"))
        }).await.map_err(srv_err)??;

        // 2. Fetch models from Fox
        let url = format!("http://{}:8080/v1/models", host_addr);
        let client = reqwest::Client::new();
        let resp = client.get(&url).send().await.map_err(|e| srv_err(&format!("Fox API unreachable: {}", e)))?;
        
        let body: serde_json::Value = resp.json().await.map_err(|e| srv_err(&format!("Invalid Fox response: {}", e)))?;
        let fox_models = body["data"].as_array().ok_or_else(|| srv_err("Invalid models list from Fox"))?;

        // 3. Update DB
        let pool_update = pool.clone();
        let host_id_for_update = host_id.clone();
        let fox_models_clone: Vec<String> = fox_models.iter().filter_map(|m| m["id"].as_str().map(|s| s.to_string())).collect();

        tokio::task::spawn_blocking(move || -> Result<(), ServerFnError> {
            let conn = pool_update.get().map_err(srv_err)?;
            
            for fox_id in &fox_models_clone {
                // Heuristic: Match by exact name, repo_id containment, or partial overlap
                let mut stmt = conn.prepare("SELECT id FROM llms WHERE host_id = ? AND provider = 'Fox' \
                    AND (model_name = ? OR repo_id LIKE ? OR instr(?, replace(model_name, '/', '_')) > 0 OR instr(repo_id, ?) > 0) \
                    ORDER BY id DESC LIMIT 1;").map_err(srv_err)?;
                
                let res: Result<Option<String>, duckdb::Error> = stmt.query_row(duckdb::params![host_id_for_update, fox_id, format!("%{}%", fox_id), fox_id, fox_id], |r| r.get(0)).optional();
                let existing_id = res.map_err(srv_err)?;

                if let Some(id) = existing_id {
                    conn.execute("UPDATE llms SET model_name = ?, status = 'online', last_synced_at = CURRENT_TIMESTAMP WHERE id = ?;", duckdb::params![fox_id, id]).map_err(srv_err)?;
                } else {
                    let id = uuid::Uuid::new_v4().to_string();
                    conn.execute(
                        "INSERT INTO llms (id, host_id, name, provider, model_name, status, download_status, last_synced_at) \
                         VALUES (?, ?, ?, 'Fox', ?, 'online', 'none', CURRENT_TIMESTAMP);",
                        duckdb::params![id, host_id_for_update, fox_id.clone(), fox_id],
                    ).map_err(srv_err)?;
                }
            }

            // Cleanup: delete older records where we now have a better replacement
            conn.execute(
                "DELETE FROM llms WHERE host_id = ? AND provider = 'Fox' AND (model_name LIKE '%/%' OR repo_id LIKE '%/%');",
                duckdb::params![host_id_for_update]
            ).map_err(srv_err)?;
            
            Ok(())
        }).await.map_err(srv_err)??;

        Ok(())
    }
    #[cfg(not(feature = "ssr"))]
    { unreachable!() }
}

#[server(DeployFox, "/api")]
pub async fn deploy_fox(host_id: String) -> Result<(), ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        use axum::extract::Extension;
        use duckdb::DuckdbConnectionManager;
        use r2d2::Pool;
        use leptos_axum::extract;
        use crate::hosts::Host;

        crate::auth::require_role(crate::auth::UserRole::Operator).await?;

        let Extension(pool) = extract::<Extension<Pool<DuckdbConnectionManager>>>()
            .await
            .map_err(|_| srv_err("Failed to extract DB pool"))?;

        // 1. Get host info
        let pool_clone = pool.clone();
        let host_id_clone = host_id.clone();
        let host = tokio::task::spawn_blocking(move || -> Result<Host, ServerFnError> {
            let conn = pool_clone.get().map_err(srv_err)?;
            let mut stmt = conn.prepare("SELECT id, name, address, port, username, auth_method, password, ssh_key, ssh_public_key, ssh_passphrase, notes, status, zfs_pool_size_gb, storage_device FROM hosts WHERE id = ?;").map_err(srv_err)?;
            let host = stmt.query_row(duckdb::params![host_id_clone], |row| Ok(Host {
                id: row.get(0)?,
                name: row.get(1)?,
                address: row.get(2)?,
                port: row.get(3)?,
                username: row.get(4)?,
                auth_method: row.get(5)?,
                password: row.get(6)?,
                ssh_key: row.get(7)?,
                ssh_public_key: row.get(8)?,
                ssh_passphrase: row.get(9)?,
                notes: row.get(10)?,
                status: row.get(11)?,
                zfs_pool_size_gb: row.get(12)?,
                storage_device: row.get(13)?,
            })).map_err(srv_err)?;
            Ok(host)
        }).await.map_err(srv_err)??;

        // 2. Identify controller and create Linstor client
        let controller = crate::hosts::get_controller_host(pool.clone()).await?
            .ok_or_else(|| srv_err("No Linstor controller found"))?;
        let _linstor_client = crate::storage::linstor::LinstorClient::new(&controller.address);

        // 3. Create a job for tracking
        let job_id = crate::jobs::create_job(pool.clone(), format!("Deploy Fox: {}", host.name)).await?;
        crate::jobs::update_job(pool.clone(), job_id.clone(), "running".into(), 10).await?;
        let _ = crate::jobs::add_job_log(pool.clone(), job_id.clone(), "Ensuring Linstor volume 'llm-models' exists...".into()).await;

        // 4. Orchestration script (Linstor definition + placement + format + mount + host-native fox)
        let host_name_for_script = host.name.clone();
        let script = format!(
            "set -ex\n\
              # 1. Optimize Storage Check: Skip Linstor if already mounted\n\
              if ! mountpoint -q /mnt/llm-models; then\n\
                echo 'Cleaning up Linstor resources...'\n\
                sudo linstor resource-definition create llm-models || true\n\
                if ! sudo linstor volume-definition list -r llm-models | grep -q ' 0 '; then\n\
                  sudo linstor volume-definition create llm-models 30G\n\
                else\n\
                  sudo linstor volume-definition set-size llm-models 0 30G || true\n\
                fi\n\
                if ! sudo linstor resource list -n {0} -r llm-models | grep -q 'llm-models'; then\n\
                  sudo linstor resource create {0} llm-models --storage-pool cloudlab_pool || \\\n\
                  sudo linstor resource create {0} llm-models --storage-pool cloudlab_pool --layer-list STORAGE\n\
                fi\n\
                DEVICE=''\n\
                for i in $(seq 1 15); do\n\
                  DEVICE=$(sudo linstor --no-utf8 volume list -n {0} -r llm-models | grep '/dev/' | awk -F'|' '{{print $7}}' | xargs) || true\n\
                  if [[ -b '$DEVICE' ]]; then break; fi\n\
                  DEVICE=$(sudo linstor --no-utf8 volume list -n {0} -r llm-models | grep '/dev/' | grep -o '/dev/[^ ]*') || true\n\
                  if [[ -b '$DEVICE' ]]; then break; fi\n\
                  sleep 2\n\
                done\n\
                if [[ ! -b '$DEVICE' ]]; then exit 1; fi\n\
                if ! sudo blkid '$DEVICE'; then sudo mkfs.ext4 '$DEVICE'; fi\n\
                sudo mkdir -p /mnt/llm-models\n\
                echo \"Mounting $DEVICE to /mnt/llm-models...\"\n\
                if ! sudo mount \"$DEVICE\" /mnt/llm-models; then\n\
                  echo \"CRITICAL: Failed to mount $DEVICE. Check dmesg.\"\n\
                  exit 1\n\
                fi\n\
              fi\n\
              sudo chown -R cloudlab:cloudlab /mnt/llm-models\n\
              \n\
              # 0. Global Clean Start\n\
              sudo systemctl stop fox || true\n\
              sudo pkill -9 fox || true\n\
              \n\
              if [ ! -f /usr/local/bin/.fox-built ]; then\n\
                if [ ! -f /tmp/fox-build/target/release/fox ]; then\n\
                  echo 'DEBUG: Step 1/5 - Installing build dependencies...'\n\
                  export DEBIAN_FRONTEND=noninteractive\n\
                  sudo apt-get install -y build-essential cmake git pkg-config libvulkan-dev clang libclang-dev\n\
                  \n\
                  if ! command -v cargo &> /dev/null; then\n\
                    echo 'DEBUG: Step 2/5 - Installing Rust toolchain...'\n\
                    curl -fsSL https://sh.rustup.rs | sh -s -- -y\n\
                  fi\n\
                  \n\
                  source $HOME/.cargo/env || source /root/.cargo/env || true\n\
                  export PATH=\"$HOME/.cargo/bin:/root/.cargo/bin:$PATH\"\n\
                  \n\
                  echo 'DEBUG: Step 3/5 - Cloning Fox and Submodules...'\n\
                  rm -rf /tmp/fox-build && mkdir -p /tmp/fox-build\n\
                  git clone --depth 1 https://github.com/ferrumox/fox /tmp/fox-build\n\
                  cd /tmp/fox-build\n\
                  git submodule update --init --recursive\n\
                  \n\
                  echo \"DEBUG: Step 4/5 - Compiling Fox with $(nproc) cores (Estimated 5-10m)...\"\n\
                  cargo build --release\n\
                  cd -\n\
                else\n\
                  echo 'DEBUG: Detected existing build in /tmp. Skipping re-compilation.'\n\
                fi\n\
                \n\
                echo 'DEBUG: Step 5/5 - Stopping service and installing build artifacts...'\n\
                sudo systemctl stop fox || true\n\
                sudo pkill -9 fox || true\n\
                sudo rm -f /usr/local/bin/fox\n\
                sudo cp /tmp/fox-build/target/release/fox /usr/local/bin/\n\
                sudo cp /tmp/fox-build/target/release/*.so /usr/local/bin/ || true\n\
                sudo touch /usr/local/bin/.fox-built\n\
              fi\n\
              \n\
              # 2. Final GPU Detection & Environment Setup\n\
              ENV_FLAGS='-E FOX_HOST=0.0.0.0 -E FOX_PORT=8080 -E FOX_LAZY=false'\n\
              if /usr/bin/nvidia-smi &> /dev/null || nvidia-smi &> /dev/null; then\n\
                echo 'NVIDIA GPU detected. Activating host-native acceleration.'\n\
                ENV_FLAGS=\"$ENV_FLAGS -E FOX_GPU_MEMORY_FRACTION=0.65 -E FOX_MAX_BATCH_SIZE=1 -E FOX_MAIN_GPU=0 -E LD_LIBRARY_PATH=/usr/local/bin:/usr/lib/x86_64-linux-gnu -E GGML_VULKAN_DEVICE=0 -E LLAMA_CUDA_FORCE_DMMV=1 -E GGML_CUDA_NO_VMM=1\"\n\
              fi\n\
              \n\
              # 3. Host-Native Process Management (systemd-run)\n\
              echo 'Cleaning up orphaned Fox processes...'\n\
              sudo pkill -9 fox || true\n\
              \n\
              echo 'Ensuring model registry symlink...'\n\
              sudo mkdir -p /root/.cache/ferrumox\n\
              sudo ln -sfn /mnt/llm-models /root/.cache/ferrumox/models\n\
              \n\
              sudo systemctl stop fox || true\n\
              sudo systemctl reset-failed fox || true\n\
              sudo systemctl daemon-reload\n\
              sudo systemd-run --unit=fox --description='Fox Inference Engine' --remain-after-exit $ENV_FLAGS /usr/local/bin/fox serve\n\
              \n\
              # 4. Verification\n\
              echo 'Verifying host-native process (waiting for port 8080)...'\n\
              for i in {{1..30}}; do\n\
                if curl -s http://localhost:8080/health &> /dev/null || curl -s http://localhost:8080/v1/models &> /dev/null; then\n\
                  break\n\
                fi\n\
                sleep 2\n\
              done\n\
              \n\
              echo 'Triggering model load (this may take up to 2m)...'\n\
              if ! curl -i -X POST http://localhost:8080/v1/chat/completions \\\n\
                 --max-time 120 \\\n\
                 -H 'Content-Type: application/json' \\\n\
                 -d '{{\"model\": \"gemma-4-E2B-it-Q4_K_M\", \"messages\": [{{\"role\": \"user\", \"content\": \"hi\"}}], \"max_tokens\": 1}}'; then\n\
                echo 'ERROR: Model load failed or timed out. Check logs below:'\n\
                sudo systemctl status fox --no-pager\n\
                sudo journalctl -u fox -n 100 --no-pager\n\
                exit 1\n\
              fi\n\
              sleep 5\n\
              echo '--- Host Process Check ---'\n\
              ps aux | grep fox | grep serve | grep -v grep\n\
              nvidia-smi\n\
              echo '--- Service Logs ---'\n\
              sudo journalctl -u fox --no-pager -n 100\n\
              echo '--- End Logs ---'",
            host_name_for_script
        );

        let pool_for_ssh = pool.clone();
        let host_id_for_ssh = host.id.clone();
        let job_id_for_ssh = job_id.clone();
        let pool_for_done = pool.clone();
        
        let _job_id_for_error = job_id.clone();
        tokio::spawn(async move {
            let job_id_inner = job_id_for_ssh.clone();
            let res = tokio::task::spawn_blocking(move || -> Result<(i32, String, String), ServerFnError> {
                let conn = pool_for_ssh.get().map_err(srv_err)?;
                let sess = crate::hosts::get_host_session_blocking(&conn, &host_id_for_ssh)?;
                
                let _ = crate::jobs::add_job_log(pool_for_ssh.clone(), job_id_inner, "Executing deployment script on host...".into());
                let (status, stdout, stderr) = crate::ssh::run_remote_script(&sess, &script, host.password.as_deref())?;
                
                Ok((status, stdout, stderr))
            }).await;

            match res {
                Ok(Ok((status, stdout, stderr))) => {
                    if !stdout.is_empty() {
                        let _ = crate::jobs::add_job_log(pool_for_done.clone(), job_id_for_ssh.clone(), format!("STDOUT LOGS ({} chars):\n{}", stdout.len(), stdout)).await;
                    }
                    if !stderr.is_empty() {
                        let _ = crate::jobs::add_job_log(pool_for_done.clone(), job_id_for_ssh.clone(), format!("STDERR LOGS ({} chars):\n{}", stderr.len(), stderr)).await;
                    }

                    if status == 0 {
                        let _ = crate::jobs::update_job(pool_for_done.clone(), job_id_for_ssh.clone(), "completed".into(), 100).await;
                        let _ = crate::jobs::add_job_log(pool_for_done.clone(), job_id_for_ssh.clone(), "Fox Inference Engine deployed successfully.".into()).await;
                    } else {
                        let _ = crate::jobs::update_job(pool_for_done.clone(), job_id_for_ssh.clone(), "failed".into(), 0).await;
                        let _ = crate::jobs::add_job_log(pool_for_done.clone(), job_id_for_ssh.clone(), format!("Fox deployment failed (exit {}).", status)).await;
                    }
                }
                Ok(Err(e)) => {
                    let _ = crate::jobs::update_job(pool_for_done.clone(), job_id_for_ssh.clone(), "failed".into(), 0).await;
                    let _ = crate::jobs::add_job_log(pool_for_done.clone(), job_id_for_ssh.clone(), format!("Error: {}", e)).await;
                }
                Err(e) => {
                    let _ = crate::jobs::update_job(pool_for_done.clone(), job_id_for_ssh.clone(), "failed".into(), 0).await;
                    let _ = crate::jobs::add_job_log(pool_for_done.clone(), job_id_for_ssh.clone(), format!("Internal Error: {}", e)).await;
                }
            }
        });

        // 3. Register the LLM service in our DB if not exists
        let pool_clone = pool.clone();
        let host_id_clone = host.id.clone();
         tokio::task::spawn_blocking(move || -> Result<(), ServerFnError> {
            let conn = pool_clone.get().map_err(srv_err)?;
            let id = uuid::Uuid::new_v4().to_string();
            conn.execute(
                "INSERT INTO llms (id, host_id, name, provider, model_name, status) \
                 VALUES (?, ?, ?, 'Fox', 'Fox Engine', 'online') \
                 ON CONFLICT (host_id, provider, model_name) DO UPDATE SET status='online';",
                duckdb::params![id, host_id_clone, "Fox Inference Engine"],
            ).map_err(srv_err)?;
            Ok(())
        }).await.map_err(srv_err)??;

        Ok(())
    }
    #[cfg(not(feature = "ssr"))]
    { unreachable!() }
}

#[allow(dead_code)]
#[derive(Debug, Serialize, Deserialize)]
struct PullStatus {
    status: String,
    digest: Option<String>,
    total: Option<u64>,
    completed: Option<u64>,
}

#[server(PullModel, "/api")]
pub async fn pull_model(host_id: String, model_repo: String) -> Result<(), ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        use axum::extract::Extension;
        use duckdb::DuckdbConnectionManager;
        use r2d2::Pool;
        use leptos_axum::extract;
        use futures::StreamExt;
        use std::collections::HashMap;
        
        crate::auth::require_role(crate::auth::UserRole::Operator).await?;

        let Extension(pool) = extract::<Extension<Pool<DuckdbConnectionManager>>>()
            .await
            .map_err(|_| srv_err("Failed to extract DB pool"))?;

        // 1. Get host IP
        let pool_clone = pool.clone();
        let host_id_for_ip = host_id.clone();
        let host_addr = tokio::task::spawn_blocking(move || -> Result<String, ServerFnError> {
            let conn = pool_clone.get().map_err(srv_err)?;
            let res: Result<Option<String>, duckdb::Error> = conn.query_row(
                "SELECT address FROM hosts WHERE id = ?;", 
                duckdb::params![host_id_for_ip], 
                |r| r.get(0)
            ).optional();
            let addr = res.map_err(srv_err)?;
            
            addr.ok_or_else(|| srv_err(&format!("Target host '{}' not found. Please ensure the host exists.", host_id_for_ip)))
        }).await.map_err(srv_err)??;

        // 2. Start pull request
        let url = format!("http://{}:8080/api/pull", host_addr);
        let client = reqwest::Client::new();
        let model_repo_clone = model_repo.clone();
        
        let resp = client.post(&url)
            .json(&serde_json::json!({ "name": model_repo_clone }))
            .send()
            .await
            .map_err(|e| srv_err(&format!("Failed to connect to Fox: {}", e)))?;

        if !resp.status().is_success() {
            let body = resp.text().await.unwrap_or_default();
            return Err(srv_err(&format!("Fox API Error: {}", body)));
        }

        // 3. Mark as downloading in our DB
        let pool_clone = pool.clone();
        let host_id_for_db = host_id.clone();
        let model_repo_for_db = model_repo.clone();
        tokio::task::spawn_blocking(move || -> Result<(), ServerFnError> {
            let conn = pool_clone.get().map_err(srv_err)?;
            let id = uuid::Uuid::new_v4().to_string();
            conn.execute(
                "INSERT INTO llms (id, host_id, name, provider, model_name, status, repo_id, download_status) \
                 VALUES (?, ?, ?, 'Fox', ?, 'offline', ?, 'downloading') \
                 ON CONFLICT (host_id, provider, model_name) DO UPDATE SET repo_id = excluded.repo_id, download_status = 'downloading';",
                duckdb::params![id, host_id_for_db, model_repo_for_db, model_repo_for_db, model_repo_for_db],
            ).map_err(srv_err)?;
            Ok(())
        }).await.map_err(srv_err)??;

        // 4. Spawn background task to track progress
        let pool_background = pool.clone();
        let host_id_background = host_id.clone();
        let model_repo_background = model_repo.clone();
        tokio::spawn(async move {
            let mut stream = resp.bytes_stream();
            let mut layers: HashMap<String, (u64, u64)> = HashMap::new();
            let mut last_update = std::time::Instant::now();
            let mut last_percent = -1;

            while let Some(chunk_res) = stream.next().await {
                let chunk = match chunk_res {
                    Ok(c) => c,
                    Err(_) => break,
                };
                
                // NDJSON parsing
                let s = String::from_utf8_lossy(&chunk);
                for line in s.lines() {
                    if let Ok(status) = serde_json::from_str::<PullStatus>(line) {
                        if let (Some(digest), Some(total), Some(completed)) = (status.digest, status.total, status.completed) {
                            layers.insert(digest, (completed, total));
                            
                            // Calculate total progress
                            let total_bytes: u64 = layers.values().map(|v| v.1).sum();
                            let completed_bytes: u64 = layers.values().map(|v| v.0).sum();
                            
                            if total_bytes > 0 {
                                let percent = (completed_bytes * 100 / total_bytes) as i32;
                                
                                // Throttle updates to DB: once per 1s OR if percentage changed significantly
                                if percent != last_percent && (last_update.elapsed().as_secs() >= 1 || (percent - last_percent).abs() >= 5) {
                                    let pool_inner = pool_background.clone();
                                    let host_id_inner = host_id_background.clone();
                                    let model_name_inner = model_repo_background.clone();
                                    let status_str = format!("downloading:{}", percent);
                                    
                                    let _ = tokio::task::spawn_blocking(move || {
                                        if let Ok(conn) = pool_inner.get() {
                                            let _ = conn.execute(
                                                "UPDATE llms SET download_status = ? WHERE host_id = ? AND provider = 'Fox' AND model_name = ?;",
                                                duckdb::params![status_str, host_id_inner, model_name_inner]
                                            );
                                        }
                                    }).await;
                                    
                                    last_percent = percent;
                                    last_update = std::time::Instant::now();
                                }
                            }
                        } else if status.status == "success" {
                            // Done!
                            let pool_inner = pool_background.clone();
                            let host_id_inner = host_id_background.clone();
                            let model_name_inner = model_repo_background.clone();
                            let _ = tokio::task::spawn_blocking(move || {
                                if let Ok(conn) = pool_inner.get() {
                                    let _ = conn.execute(
                                        "UPDATE llms SET download_status = 'none', status = 'offline' WHERE host_id = ? AND provider = 'Fox' AND model_name = ?;",
                                        duckdb::params![host_id_inner, model_name_inner]
                                    );
                                }
                            }).await;
                            return;
                        }
                    }
                }
            }
            
            // If we exited the loop without "success", mark as failed/none
            let pool_inner = pool_background.clone();
            let host_id_inner = host_id_background.clone();
            let model_name_inner = model_repo_background.clone();
            let _ = tokio::task::spawn_blocking(move || {
                if let Ok(conn) = pool_inner.get() {
                    let _ = conn.execute(
                        "UPDATE llms SET download_status = 'none' WHERE host_id = ? AND provider = 'Fox' AND model_name = ? AND download_status LIKE 'downloading%';",
                        duckdb::params![host_id_inner, model_name_inner]
                    );
                }
            }).await;
        });

        Ok(())
    }
    #[cfg(not(feature = "ssr"))]
    { unreachable!() }
}

#[server(SearchHFModels, "/api")]
pub async fn search_hf_models(query: String) -> Result<Vec<HFModel>, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        let encoded_query = urlencoding::encode(&query);
        let url = format!("https://huggingface.co/api/models?search={}&filter=gguf&sort=downloads&direction=-1&limit=20", encoded_query);
        
        let client = reqwest::Client::new();
        let resp = client.get(url)
            .header("User-Agent", "CloudLab/1.0")
            .send()
            .await
            .map_err(|e| srv_err(&format!("HF Search failed: {}", e)))?;
            
        let models: Vec<HFModel> = resp.json()
            .await
            .map_err(|e| srv_err(&format!("Failed to parse HF response: {}", e)))?;
            
        Ok(models)
    }
    #[cfg(not(feature = "ssr"))]
    {
        let _ = query;
        unreachable!()
    }
}
