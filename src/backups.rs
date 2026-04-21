use leptos::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct BackupConfig {
    pub id: String,
    pub name: String,
    pub source_type: String, // VM, Container, Database, External
    pub source_id: String,
    pub destination: String,
    pub status: String,
    pub last_run: Option<String>,
    pub schedule: Option<String>,
}

#[cfg(feature = "ssr")]
use duckdb::params;

#[cfg(feature = "ssr")]
use crate::auth::srv_err;

// ─── List ────────────────────────────────────────────────────────────────────

#[server(ListBackups, "/api")]
pub async fn list_backups() -> Result<Vec<BackupConfig>, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        use axum::extract::Extension;
        use duckdb::DuckdbConnectionManager;
        use leptos_axum::extract;
        use r2d2::Pool;

        let Extension(pool) = extract::<Extension<Pool<DuckdbConnectionManager>>>()
            .await
            .map_err(|_| srv_err("Failed to extract DB pool"))?;

        let backups = tokio::task::spawn_blocking(move || -> Result<Vec<BackupConfig>, ServerFnError> {
            let conn = pool.get().map_err(srv_err)?;
            let mut stmt = conn
                .prepare(
                    "SELECT id, name, source_type, source_id, destination, status, strftime(last_run, '%Y-%m-%d %H:%M:%S'), schedule \
                     FROM backups \
                     ORDER BY name ASC;",
                )
                .map_err(srv_err)?;
            let mut rows = stmt.query(params![]).map_err(srv_err)?;
            let mut backups = Vec::new();
            while let Some(row) = rows.next().map_err(srv_err)? {
                backups.push(BackupConfig {
                    id: row.get::<_, String>(0).map_err(srv_err)?,
                    name: row.get::<_, String>(1).map_err(srv_err)?,
                    source_type: row.get::<_, String>(2).map_err(srv_err)?,
                    source_id: row.get::<_, String>(3).map_err(srv_err)?,
                    destination: row.get::<_, String>(4).map_err(srv_err)?,
                    status: row.get::<_, String>(5).map_err(srv_err)?,
                    last_run: row.get::<_, Option<String>>(6).map_err(srv_err)?,
                    schedule: row.get::<_, Option<String>>(7).map_err(srv_err)?,
                });
            }
            Ok(backups)
        })
        .await
        .map_err(srv_err)??;

        Ok(backups)
    }
    #[cfg(not(feature = "ssr"))]
    { unreachable!() }
}

// ─── Create ──────────────────────────────────────────────────────────────────

#[server(CreateBackupConfig, "/api")]
pub async fn create_backup_config(
    name: String,
    source_type: String,
    source_id: String,
    destination: String,
    schedule: Option<String>,
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
            let id = uuid::Uuid::new_v4().to_string();
            conn.execute(
                "INSERT INTO backups (id, name, source_type, source_id, destination, status, last_run, schedule) \
                 VALUES (?, ?, ?, ?, ?, 'idle', NULL, ?);",
                params![id, name, source_type, source_id, destination, schedule],
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

// ─── Run Backup ──────────────────────────────────────────────────────────────

#[server(RunBackupNow, "/api")]
pub async fn run_backup_now(id: String) -> Result<(), ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        crate::auth::require_role(crate::auth::UserRole::Operator).await?;
        use axum::extract::Extension;
        use duckdb::DuckdbConnectionManager;
        use leptos_axum::extract;
        use r2d2::Pool;
        // use crate::hosts::get_host_session_blocking;

        let Extension(pool) = extract::<Extension<Pool<DuckdbConnectionManager>>>()
            .await
            .map_err(|_| srv_err("Failed to extract DB pool"))?;

        // 1. Get config
        let config = tokio::task::spawn_blocking({
            let pool = pool.clone();
            let id = id.clone();
            move || -> Result<BackupConfig, ServerFnError> {
                let conn = pool.get().map_err(srv_err)?;
                let mut stmt = conn.prepare("SELECT id, name, source_type, source_id, destination, status, strftime(last_run, '%Y-%m-%d %H:%M:%S'), schedule FROM backups WHERE id=?;").map_err(srv_err)?;
                let res = stmt.query_row(params![id], |row| Ok(BackupConfig {
                    id: row.get::<_, String>(0)?,
                    name: row.get::<_, String>(1)?,
                    source_type: row.get::<_, String>(2)?,
                    source_id: row.get::<_, String>(3)?,
                    destination: row.get::<_, String>(4)?,
                    status: row.get::<_, String>(5)?,
                    last_run: row.get::<_, Option<String>>(6)?,
                    schedule: row.get::<_, Option<String>>(7)?,
                })).map_err(srv_err)?;
                Ok(res)
            }
        })
        .await
        .map_err(srv_err)??;

        // Update status to running
        tokio::task::spawn_blocking({
            let pool = pool.clone();
            let id = id.clone();
            move || -> Result<(), ServerFnError> {
                let conn = pool.get().map_err(srv_err)?;
                conn.execute("UPDATE backups SET status='running' WHERE id=?;", params![id]).map_err(srv_err)?;
                Ok(())
            }
        }).await.map_err(srv_err)??;

        // 2. Execute backup logic (Simplified demo logic)
        // In a real app, this would be much more complex.
        match config.source_type.as_str() {
            "Database" => {
                // Implement DB specific dumps here...
            },
            "Container" => {
                // Implement container volume archiving...
            },
            "VM" => {
                // VM snapshot...
            },
            "External" => {
                // Rsync...
            },
            _ => return Err(srv_err("Unsupported source type")),
        }

        // 3. Mark as idle/finished and update last_run
        tokio::task::spawn_blocking(move || -> Result<(), ServerFnError> {
            let conn = pool.get().map_err(srv_err)?;
            conn.execute("UPDATE backups SET status='idle', last_run=CURRENT_TIMESTAMP WHERE id=?;", params![id]).map_err(srv_err)?;
            Ok(())
        }).await.map_err(srv_err)??;

        Ok(())
    }
    #[cfg(not(feature = "ssr"))]
    { unreachable!() }
}

// ─── Delete ──────────────────────────────────────────────────────────────────

#[server(DeleteBackupConfig, "/api")]
pub async fn delete_backup_config(id: String) -> Result<(), ServerFnError> {
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
            conn.execute("DELETE FROM backups WHERE id=?;", params![id]).map_err(srv_err)?;
            Ok(())
        })
        .await
        .map_err(srv_err)??;

        Ok(())
    }
    #[cfg(not(feature = "ssr"))]
    { unreachable!() }
}
