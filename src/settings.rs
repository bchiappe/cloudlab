use leptos::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct GlobalSettings {
    pub ha_enabled: bool,
    pub sync_interval: i32, // in seconds
    pub cluster_name: String,
    pub secondary_node_ip: String,
}

#[cfg(feature = "ssr")]
use crate::auth::srv_err;

#[cfg(feature = "ssr")]
use duckdb::params;

#[server(GetSettings, "/api")]
pub async fn get_settings() -> Result<GlobalSettings, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        use axum::extract::Extension;
        use duckdb::DuckdbConnectionManager;
        use r2d2::Pool;
        use leptos_axum::extract;

        let Extension(pool) = extract::<Extension<Pool<DuckdbConnectionManager>>>()
            .await
            .map_err(srv_err)?;

        let settings = tokio::task::spawn_blocking(move || -> Result<GlobalSettings, ServerFnError> {
            let conn = pool.get().map_err(srv_err)?;
            let mut stmt = conn.prepare("SELECT key, value FROM global_settings;").map_err(srv_err)?;
            let mut rows = stmt.query(params![]).map_err(srv_err)?;
            
            let mut ha_enabled = false;
            let mut sync_interval = 30;
            let mut cluster_name = String::new();
            let mut secondary_node_ip = String::new();

            while let Some(row) = rows.next().map_err(srv_err)? {
                let key: String = row.get(0)?;
                let value: String = row.get(1)?;
                match key.as_str() {
                    "ha_enabled" => ha_enabled = value == "true",
                    "sync_interval" => sync_interval = value.parse().unwrap_or(30),
                    "cluster_name" => cluster_name = value,
                    "secondary_node_ip" => secondary_node_ip = value,
                    _ => {}
                }
            }

            Ok(GlobalSettings {
                ha_enabled,
                sync_interval,
                cluster_name,
                secondary_node_ip,
            })
        }).await.map_err(srv_err)??;

        Ok(settings)
    }
    #[cfg(not(feature = "ssr"))]
    { unreachable!() }
}

#[server(UpdateSettings, "/api")]
pub async fn update_settings(settings: GlobalSettings) -> Result<(), ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        crate::auth::require_role(crate::auth::UserRole::Admin).await?;
        use axum::extract::Extension;
        use duckdb::DuckdbConnectionManager;
        use r2d2::Pool;
        use leptos_axum::extract;

        let Extension(pool) = extract::<Extension<Pool<DuckdbConnectionManager>>>()
            .await
            .map_err(srv_err)?;

        tokio::task::spawn_blocking(move || -> Result<(), ServerFnError> {
            let conn = pool.get().map_err(srv_err)?;
            
            let updates = [
                ("ha_enabled", settings.ha_enabled.to_string()),
                ("sync_interval", settings.sync_interval.to_string()),
                ("cluster_name", settings.cluster_name),
                ("secondary_node_ip", settings.secondary_node_ip),
            ];

            for (key, val) in updates {
                conn.execute(
                    "INSERT OR REPLACE INTO global_settings (key, value) VALUES (?, ?);",
                    params![key, val]
                ).map_err(srv_err)?;
            }
            
            Ok(())
        }).await.map_err(srv_err)??;

        Ok(())
    }
    #[cfg(not(feature = "ssr"))]
    { unreachable!() }
}
