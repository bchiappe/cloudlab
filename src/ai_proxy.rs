#[cfg(feature = "ssr")]
pub mod srv {
    use axum::{
        body::Body,
        extract::{Path, State, Extension},
        http::{Request, StatusCode, HeaderMap},
        response::{IntoResponse, Response},
    };
    use leptos::prelude::*;
    use r2d2::Pool;
    use duckdb::DuckdbConnectionManager;
    use bcrypt::verify;
    
    use sync_wrapper::SyncStream;

    pub async fn ai_proxy_handler(
        State(_leptos_options): State<LeptosOptions>,
        Extension(pool): Extension<Pool<DuckdbConnectionManager>>,
        headers: HeaderMap,
        Path(path): Path<String>,
        req: Request<Body>,
    ) -> impl IntoResponse {
        // ... (auth and target identification same as before) ...
        // Re-implementing the middle part to ensure all variables are in scope if I use a smaller chunk,
        // but I'll just replace the whole function body or the relevant parts.

        // 1. Auth check
        let auth_header = match headers.get("Authorization") {
            Some(h) => h.to_str().unwrap_or(""),
            None => return (StatusCode::UNAUTHORIZED, "Missing Authorization header").into_response(),
        };

        if !auth_header.starts_with("Bearer ") {
            return (StatusCode::UNAUTHORIZED, "Invalid Authorization format").into_response();
        }

        let key = &auth_header[7..];
        let prefix = if key.len() > 8 { &key[0..8] } else { key };

        let pool_clone = pool.clone();
        let prefix_clone = prefix.to_string();
        let key_clone = key.to_string();

        let is_valid = tokio::task::spawn_blocking(move || -> bool {
            let conn = match pool_clone.get() {
                Ok(c) => c,
                Err(_) => return false,
            };
            let mut stmt = match conn.prepare("SELECT hashed_key FROM api_keys WHERE key_prefix = ?;") {
                Ok(s) => s,
                Err(_) => return false,
            };
            let mut rows = match stmt.query(duckdb::params![prefix_clone]) {
                Ok(r) => r,
                Err(_) => return false,
            };
            
            while let Ok(Some(row)) = rows.next() {
                let hashed: String = row.get(0).unwrap_or_default();
                if verify(&key_clone, &hashed).unwrap_or(false) {
                    return true;
                }
            }
            false
        }).await.unwrap_or(false);

        if !is_valid {
            return (StatusCode::UNAUTHORIZED, "Invalid API Key").into_response();
        }

        // 2. Identify Target
        let pool_for_target = pool.clone();
        let target_host = tokio::task::spawn_blocking(move || -> Option<String> {
            let conn = pool_for_target.get().ok()?;
            let mut stmt = conn.prepare("SELECT h.address FROM llms l JOIN hosts h ON l.host_id = h.id WHERE l.status = 'online' LIMIT 1;").ok()?;
            stmt.query_row([], |row| row.get(0)).ok()
        }).await.unwrap_or(None);

        let host_addr = match target_host {
            Some(addr) => addr,
            None => return (StatusCode::SERVICE_UNAVAILABLE, "No online LLM hosts found").into_response(),
        };

        // 3. Proxy request
        let target_url = format!("http://{}:8080/v1/{}", host_addr, path);
        let client = reqwest::Client::new();
        
        let (parts, body) = req.into_parts();
        
        // Use sync_wrapper to satisfy reqwest::Body::wrap_stream Sync requirement
        let sync_body = SyncStream::new(body.into_data_stream());
        let proxy_body = reqwest::Body::wrap_stream(sync_body);

        let proxy_req = client.post(&target_url)
            .headers(parts.headers)
            .body(proxy_body);

        let proxy_resp = match proxy_req.send().await {
            Ok(res) => res,
            Err(e) => return (StatusCode::BAD_GATEWAY, format!("Proxy failed: {}", e)).into_response(),
        };

        let mut res_builder = Response::builder()
            .status(proxy_resp.status());
        
        for (k, v) in proxy_resp.headers().iter() {
            res_builder = res_builder.header(k, v);
        }

        // Forward the stream back to Axum
        let stream = proxy_resp.bytes_stream();
        res_builder.body(Body::from_stream(stream)).unwrap()
    }
}
