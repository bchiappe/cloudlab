use leptos::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct EnvVar {
    pub key: String,
    pub value: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Volume {
    pub host_path: String,
    pub container_path: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PortMapping {
    pub host_port: i32,
    pub container_port: i32,
    pub protocol: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Container {
    pub id: String,
    pub host_id: String,
    pub host_name: String,
    pub name: String,
    pub image: String,
    pub status: String,
    pub cpu_limit: f64,
    pub memory_limit_mb: i32,
    pub env_vars: Vec<EnvVar>,
    pub volumes: Vec<Volume>,
    pub ports: Vec<PortMapping>,
}

#[cfg(feature = "ssr")]
use duckdb::params;

#[cfg(feature = "ssr")]
use crate::auth::srv_err;

// ─── List ────────────────────────────────────────────────────────────────────

#[server(ListContainers, "/api")]
pub async fn list_containers() -> Result<Vec<Container>, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        use axum::extract::Extension;
        use duckdb::DuckdbConnectionManager;
        use leptos_axum::extract;
        use r2d2::Pool;

        let Extension(pool) = extract::<Extension<Pool<DuckdbConnectionManager>>>()
            .await
            .map_err(|_| srv_err("Failed to extract DB pool"))?;

        let containers = tokio::task::spawn_blocking(move || -> Result<Vec<Container>, ServerFnError> {
            let conn = pool.get().map_err(srv_err)?;
            let mut stmt = conn
                .prepare(
                    "SELECT c.id, c.host_id, h.name as host_name, c.name, c.image, c.status, c.cpu_limit, c.memory_limit_mb, c.env_vars, c.volumes, c.ports \
                     FROM containers c \
                     LEFT JOIN hosts h ON c.host_id = h.id \
                     ORDER BY c.name ASC;",
                )
                .map_err(srv_err)?;
            let mut rows = stmt.query(params![]).map_err(srv_err)?;
            let mut containers = Vec::new();
            while let Some(row) = rows.next().map_err(srv_err)? {
                let env_json: String = row.get::<_, String>(8).unwrap_or_else(|_| "[]".into());
                let vol_json: String = row.get::<_, String>(9).unwrap_or_else(|_| "[]".into());
                let port_json: String = row.get::<_, String>(10).unwrap_or_else(|_| "[]".into());
                containers.push(Container {
                    id: row.get::<_, String>(0).map_err(srv_err)?,
                    host_id: row.get::<_, String>(1).map_err(srv_err).unwrap_or_default(),
                    host_name: row.get::<_, String>(2).map_err(srv_err).unwrap_or_else(|_| "Unknown".into()),
                    name: row.get::<_, String>(3).map_err(srv_err)?,
                    image: row.get::<_, String>(4).map_err(srv_err)?,
                    status: row.get::<_, String>(5).map_err(srv_err)?,
                    cpu_limit: row.get::<_, f64>(6).map_err(srv_err)?,
                    memory_limit_mb: row.get::<_, i32>(7).map_err(srv_err)?,
                    env_vars: serde_json::from_str(&env_json).unwrap_or_default(),
                    volumes: serde_json::from_str(&vol_json).unwrap_or_default(),
                    ports: serde_json::from_str(&port_json).unwrap_or_default(),
                });
            }
            Ok(containers)
        })
        .await
        .map_err(srv_err)??;

        Ok(containers)
    }
    #[cfg(not(feature = "ssr"))]
    { unreachable!() }
}

// ─── Create ──────────────────────────────────────────────────────────────────

#[server(CreateContainer, "/api")]
pub async fn create_container(
    host_id: String,
    name: String,
    image: String,
    cpu_limit: f64,
    memory_limit_mb: i32,
    env_vars: String,
    volumes: String,
    ports: String,
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
                "INSERT INTO containers (id, host_id, name, image, status, cpu_limit, memory_limit_mb, env_vars, volumes, ports) \
                 VALUES (?, ?, ?, ?, 'stopped', ?, ?, ?, ?, ?);",
                params![id, host_id, name, image, cpu_limit, memory_limit_mb, env_vars, volumes, ports],
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

// ─── Update ──────────────────────────────────────────────────────────────────

#[server(UpdateContainer, "/api")]
pub async fn update_container(
    id: String,
    host_id: String,
    name: String,
    image: String,
    cpu_limit: f64,
    memory_limit_mb: i32,
    env_vars: String,
    volumes: String,
    ports: String,
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
                "UPDATE containers SET host_id=?, name=?, image=?, cpu_limit=?, memory_limit_mb=?, env_vars=?, volumes=?, ports=? \
                 WHERE id=?;",
                params![host_id, name, image, cpu_limit, memory_limit_mb, env_vars, volumes, ports, id],
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

#[server(DeleteContainer, "/api")]
pub async fn delete_container(id: String) -> Result<(), ServerFnError> {
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
            conn.execute("DELETE FROM containers WHERE id=?;", params![id])
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

// ─── Toggle Status ────────────────────────────────────────────────────────────

#[server(ToggleContainerStatus, "/api")]
pub async fn toggle_container_status(id: String) -> Result<String, ServerFnError> {
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

        let new_status = tokio::task::spawn_blocking(move || -> Result<String, ServerFnError> {
            let conn = pool.get().map_err(srv_err)?;
            
            // Get current status
            let mut stmt = conn.prepare("SELECT status FROM containers WHERE id=?;").map_err(srv_err)?;
            let current_status: String = stmt.query_row(params![id], |row| row.get(0)).map_err(srv_err)?;
            
            let next_status = if current_status == "running" { "stopped" } else { "running" };
            
            conn.execute("UPDATE containers SET status=? WHERE id=?;", params![next_status, id]).map_err(srv_err)?;
            
            Ok(next_status.to_string())
        })
        .await
        .map_err(srv_err)??;

        Ok(new_status)
    }
    #[cfg(not(feature = "ssr"))]
    { unreachable!() }
}
