use leptos::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ManagedDatabase {
    pub id: String,
    pub host_id: String,
    pub host_name: String,
    pub name: String,
    pub db_type: String, // mysql, postgres, mariadb, mssql, oracle, mongodb
    pub port: i32,
    pub status: String,
    pub root_password: Option<String>,
    pub user_password: Option<String>,
}

#[cfg(feature = "ssr")]
use duckdb::params;

#[cfg(feature = "ssr")]
use crate::auth::srv_err;

// ─── List ────────────────────────────────────────────────────────────────────

#[server(ListDatabases, "/api")]
pub async fn list_databases() -> Result<Vec<ManagedDatabase>, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        use axum::extract::Extension;
        use duckdb::DuckdbConnectionManager;
        use leptos_axum::extract;
        use r2d2::Pool;

        let Extension(pool) = extract::<Extension<Pool<DuckdbConnectionManager>>>()
            .await
            .map_err(|_| srv_err("Failed to extract DB pool"))?;

        let databases = tokio::task::spawn_blocking(move || -> Result<Vec<ManagedDatabase>, ServerFnError> {
            let conn = pool.get().map_err(srv_err)?;
            let mut stmt = conn
                .prepare(
                    "SELECT d.id, d.host_id, h.name as host_name, d.name, d.db_type, d.port, d.status, d.root_password, d.user_password \
                     FROM databases d \
                     LEFT JOIN hosts h ON d.host_id = h.id \
                     ORDER BY d.name ASC;",
                )
                .map_err(srv_err)?;
            let mut rows = stmt.query(params![]).map_err(srv_err)?;
            let mut databases = Vec::new();
            while let Some(row) = rows.next().map_err(srv_err)? {
                databases.push(ManagedDatabase {
                    id: row.get::<_, String>(0).map_err(srv_err)?,
                    host_id: row.get::<_, String>(1).map_err(srv_err).unwrap_or_default(),
                    host_name: row.get::<_, String>(2).map_err(srv_err).unwrap_or_else(|_| "Unknown".into()),
                    name: row.get::<_, String>(3).map_err(srv_err)?,
                    db_type: row.get::<_, String>(4).map_err(srv_err)?,
                    port: row.get::<_, i32>(5).map_err(srv_err)?,
                    status: row.get::<_, String>(6).map_err(srv_err)?,
                    root_password: row.get::<_, Option<String>>(7).map_err(srv_err)?,
                    user_password: row.get::<_, Option<String>>(8).map_err(srv_err)?,
                });
            }
            Ok(databases)
        })
        .await
        .map_err(srv_err)??;

        Ok(databases)
    }
    #[cfg(not(feature = "ssr"))]
    { unreachable!() }
}

// ─── Deploy ──────────────────────────────────────────────────────────────────

#[server(DeployDatabase, "/api")]
pub async fn deploy_database(
    host_id: String,
    name: String,
    db_type: String,
    port: i32,
    root_password: String,
) -> Result<(), ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        crate::auth::require_role(crate::auth::UserRole::Operator).await?;
        use axum::extract::Extension;
        use duckdb::DuckdbConnectionManager;
        use leptos_axum::extract;
        use r2d2::Pool;
        use crate::hosts::get_host_session_blocking;

        let Extension(pool) = extract::<Extension<Pool<DuckdbConnectionManager>>>()
            .await
            .map_err(|_| srv_err("Failed to extract DB pool"))?;

        // 1. Save to database
        let id = uuid::Uuid::new_v4().to_string();
        let db_name = name.clone();
        let db_type_clone = db_type.clone();
        let root_pwd = root_password.clone();

        tokio::task::spawn_blocking({
            let pool = pool.clone();
            let id = id.clone();
            let host_id = host_id.clone();
            move || -> Result<(), ServerFnError> {
                let conn = pool.get().map_err(srv_err)?;
                conn.execute(
                    "INSERT INTO databases (id, host_id, name, db_type, port, status, root_password) \
                     VALUES (?, ?, ?, ?, ?, 'provisioning', ?);",
                    params![id, host_id, db_name, db_type_clone, port, root_pwd],
                )
                .map_err(srv_err)?;
                Ok(())
            }
        })
        .await
        .map_err(srv_err)??;

        // 2. Orchestrate Docker on the host
        let docker_cmd = match db_type.as_str() {
            "mysql" => format!(
                "docker run -d --name {} -p {}:3306 -e MYSQL_ROOT_PASSWORD={} mysql:latest",
                name, port, root_password
            ),
            "postgres" => format!(
                "docker run -d --name {} -p {}:5432 -e POSTGRES_PASSWORD={} postgres:latest",
                name, port, root_password
            ),
            "mariadb" => format!(
                "docker run -d --name {} -p {}:3306 -e MARIADB_ROOT_PASSWORD={} mariadb:latest",
                name, port, root_password
            ),
            "mssql" => format!(
                "docker run -d --name {} -p {}:1433 -e ACCEPT_EULA=Y -e MSSQL_SA_PASSWORD={} mcr.microsoft.com/mssql/server:2022-latest",
                name, port, root_password
            ),
            "oracle" => format!(
                "docker run -d --name {} -p {}:1521 -e ORACLE_PWD={} container-registry.oracle.com/database/free:latest",
                name, port, root_password
            ),
            "mongodb" => format!(
                "docker run -d --name {} -p {}:27017 -e MONGO_INITDB_ROOT_USERNAME=admin -e MONGO_INITDB_ROOT_PASSWORD={} mongo:latest",
                name, port, root_password
            ),
            _ => return Err(srv_err("Unsupported database type")),
        };

        tokio::task::spawn_blocking({
            let pool = pool.clone();
            let host_id = host_id.clone();
            let id = id.clone();
            move || -> Result<(), ServerFnError> {
                let conn = pool.get().map_err(srv_err)?;
                let sess = get_host_session_blocking(&conn, &host_id).map_err(srv_err)?;
                let mut channel = sess.channel_session().map_err(srv_err)?;
                channel.exec(&docker_cmd).map_err(srv_err)?;
                
                // Update status to online
                let conn = pool.get().map_err(srv_err)?;
                conn.execute("UPDATE databases SET status='online' WHERE id=?;", params![id])
                    .map_err(srv_err)?;
                Ok(())
            }
        })
        .await
        .map_err(srv_err)??;

        Ok(())
    }
    #[cfg(not(feature = "ssr"))]
    { unreachable!() }
}

// ─── Delete ──────────────────────────────────────────────────────────────────

#[server(DeleteDatabase, "/api")]
pub async fn delete_database(id: String) -> Result<(), ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        crate::auth::require_role(crate::auth::UserRole::Operator).await?;
        use axum::extract::Extension;
        use duckdb::DuckdbConnectionManager;
        use leptos_axum::extract;
        use r2d2::Pool;
        use crate::hosts::get_host_session_blocking;

        let Extension(pool) = extract::<Extension<Pool<DuckdbConnectionManager>>>()
            .await
            .map_err(|_| srv_err("Failed to extract DB pool"))?;

        let (host_id, name) = tokio::task::spawn_blocking({
            let pool = pool.clone();
            let id = id.clone();
            move || -> Result<(String, String), ServerFnError> {
                let conn = pool.get().map_err(srv_err)?;
                let mut stmt = conn.prepare("SELECT host_id, name FROM databases WHERE id=?;").map_err(srv_err)?;
                let res = stmt.query_row(params![id], |row| Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))).map_err(srv_err)?;
                Ok(res)
            }
        })
        .await
        .map_err(srv_err)??;

        // Remove Docker container
        tokio::task::spawn_blocking({
            let pool = pool.clone();
            let host_id = host_id.clone();
            move || -> Result<(), ServerFnError> {
                let conn = pool.get().map_err(srv_err)?;
                if let Ok(sess) = get_host_session_blocking(&conn, &host_id) {
                    let docker_cmd = format!("docker rm -f {}", name);
                    if let Ok(mut channel) = sess.channel_session() {
                        let _ = channel.exec(&docker_cmd);
                    }
                }
                Ok(())
            }
        })
        .await
        .map_err(srv_err)??;

        // Delete from DB
        tokio::task::spawn_blocking(move || -> Result<(), ServerFnError> {
            let conn = pool.get().map_err(srv_err)?;
            conn.execute("DELETE FROM databases WHERE id=?;", params![id])
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
