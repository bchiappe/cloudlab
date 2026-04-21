use leptos::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ApiKey {
    pub id: String,
    pub label: String,
    pub key_prefix: String,
    pub created_at: String,
}

#[cfg(feature = "ssr")]
use crate::auth::srv_err;
#[cfg(feature = "ssr")]
use duckdb::params;

// ─── List ────────────────────────────────────────────────────────────────────

#[server(ListApiKeys, "/api")]
pub async fn list_api_keys() -> Result<Vec<ApiKey>, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        use axum::extract::Extension;
        use duckdb::DuckdbConnectionManager;
        use leptos_axum::extract;
        use r2d2::Pool;

        let Extension(pool) = extract::<Extension<Pool<DuckdbConnectionManager>>>()
            .await
            .map_err(|_| srv_err("Failed to extract DB pool"))?;

        let keys = tokio::task::spawn_blocking(move || -> Result<Vec<ApiKey>, ServerFnError> {
            let conn = pool.get().map_err(srv_err)?;
            let mut stmt = conn
                .prepare("SELECT id, label, key_prefix, CAST(created_at AS VARCHAR) FROM api_keys ORDER BY created_at DESC;")
                .map_err(srv_err)?;
            let mut rows = stmt.query(params![]).map_err(srv_err)?;
            let mut keys = Vec::new();
            while let Some(row) = rows.next().map_err(srv_err)? {
                keys.push(ApiKey {
                    id: row.get::<_, String>(0).map_err(srv_err)?,
                    label: row.get::<_, String>(1).map_err(srv_err)?,
                    key_prefix: row.get::<_, String>(2).map_err(srv_err)?,
                    created_at: row.get::<_, String>(3).map_err(srv_err)?,
                });
            }
            Ok(keys)
        })
        .await
        .map_err(srv_err)??;

        Ok(keys)
    }
    #[cfg(not(feature = "ssr"))]
    { unreachable!() }
}

// ─── Generate ────────────────────────────────────────────────────────────────

#[server(GenerateApiKey, "/api")]
pub async fn generate_api_key(label: String) -> Result<String, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        use axum::extract::Extension;
        use duckdb::DuckdbConnectionManager;
        use leptos_axum::extract;
        use r2d2::Pool;
        use bcrypt::{hash, DEFAULT_COST};

        crate::auth::require_role(crate::auth::UserRole::Operator).await?;

        let Extension(pool) = extract::<Extension<Pool<DuckdbConnectionManager>>>()
            .await
            .map_err(|_| srv_err("Failed to extract DB pool"))?;

        // Generate a random key
        let raw_key = format!("sk-{}", uuid::Uuid::new_v4());
        let prefix = if raw_key.len() > 8 { &raw_key[0..8] } else { &raw_key };
        let hashed = hash(&raw_key, DEFAULT_COST).map_err(srv_err)?;
        let id = uuid::Uuid::new_v4().to_string();

        let label_clone = label.clone();
        let prefix_clone = prefix.to_string();
        let hashed_clone = hashed.clone();
        let id_clone = id.clone();

        tokio::task::spawn_blocking(move || -> Result<(), ServerFnError> {
            let conn = pool.get().map_err(srv_err)?;
            conn.execute(
                "INSERT INTO api_keys (id, label, key_prefix, hashed_key) VALUES (?, ?, ?, ?);",
                params![id_clone, label_clone, prefix_clone, hashed_clone],
            )
            .map_err(srv_err)?;
            Ok(())
        })
        .await
        .map_err(srv_err)??;

        // Return the raw key to the user (once!)
        Ok(raw_key)
    }
    #[cfg(not(feature = "ssr"))]
    { unreachable!() }
}

// ─── Delete ──────────────────────────────────────────────────────────────────

#[server(DeleteApiKey, "/api")]
pub async fn delete_api_key(id: String) -> Result<(), ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        use axum::extract::Extension;
        use duckdb::DuckdbConnectionManager;
        use leptos_axum::extract;
        use r2d2::Pool;

        crate::auth::require_role(crate::auth::UserRole::Operator).await?;

        let Extension(pool) = extract::<Extension<Pool<DuckdbConnectionManager>>>()
            .await
            .map_err(|_| srv_err("Failed to extract DB pool"))?;

        tokio::task::spawn_blocking(move || -> Result<(), ServerFnError> {
            let conn = pool.get().map_err(srv_err)?;
            conn.execute("DELETE FROM api_keys WHERE id=?;", params![id])
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
