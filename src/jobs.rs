use leptos::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Job {
    pub id: String,
    pub name: String,
    pub status: String, // pending, running, completed, failed
    pub progress: i32,
    pub started_at: String,
    pub finished_at: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct JobLog {
    pub id: String,
    pub job_id: String,
    pub message: String,
    pub timestamp: String,
}

#[cfg(feature = "ssr")]
use duckdb::params;

#[cfg(feature = "ssr")]
use crate::auth::srv_err;

#[cfg(feature = "ssr")]
pub async fn create_job(
    pool: r2d2::Pool<duckdb::DuckdbConnectionManager>,
    name: String,
) -> Result<String, ServerFnError> {
    leptos::logging::log!("DEBUG: Creating job with name: {}", name);
    let job_id = uuid::Uuid::new_v4().to_string();
    let name_clone = name.clone();
    let job_id_clone = job_id.clone();

    let res = tokio::task::spawn_blocking(move || -> Result<(), ServerFnError> {
        let conn = pool.get().map_err(srv_err)?;
        leptos::logging::log!("DEBUG: Got DB connection for job {}", job_id_clone);
        conn.execute(
            "INSERT INTO jobs (id, name, status, progress, started_at) VALUES (?, ?, ?, ?, CURRENT_TIMESTAMP);",
            params![job_id_clone, name_clone, "pending", 0],
        ).map_err(srv_err)?;
        leptos::logging::log!("DEBUG: Inserted job {} into DB", job_id_clone);
        
        // Count jobs in DB
        if let Ok(count) = conn.query_row("SELECT count(*) FROM jobs", params![], |r| r.get::<_, i64>(0)) {
            leptos::logging::log!("DEBUG: Total jobs in DB: {}", count);
        }
        
        Ok(())
    }).await;
    
    match res {
        Ok(Ok(_)) => {
            leptos::logging::log!("DEBUG: Job {} created successfully", job_id);
            Ok(job_id)
        }
        Ok(Err(e)) => {
            leptos::logging::log!("DEBUG: Inner error creating job: {:?}", e);
            Err(e)
        }
        Err(e) => {
            leptos::logging::log!("DEBUG: Spawn blocking error creating job: {:?}", e);
            Err(srv_err(format!("Spawn error: {}", e)))
        }
    }
}

#[cfg(feature = "ssr")]
pub async fn update_job(
    pool: r2d2::Pool<duckdb::DuckdbConnectionManager>,
    id: String,
    status: String,
    progress: i32,
) -> Result<(), ServerFnError> {
    leptos::logging::log!("DEBUG: Updating job {} to status: {}, progress: {}", id, status, progress);
    let id_clone = id.clone();
    let status_clone = status.clone();
    
    let res = tokio::task::spawn_blocking(move || -> Result<(), ServerFnError> {
        let conn = pool.get().map_err(srv_err)?;
        if status_clone == "completed" || status_clone == "failed" {
            conn.execute(
                "UPDATE jobs SET status=?, progress=?, finished_at=CURRENT_TIMESTAMP WHERE id=?;",
                params![status_clone, progress, id_clone],
            ).map_err(srv_err)?;
        } else {
            conn.execute(
                "UPDATE jobs SET status=?, progress=? WHERE id=?;",
                params![status_clone, progress, id_clone],
            ).map_err(srv_err)?;
        }
        Ok(())
    }).await;

    match res {
        Ok(Ok(_)) => {
            leptos::logging::log!("DEBUG: Job {} updated successfully", id);
            Ok(())
        }
        Ok(Err(e)) => {
            leptos::logging::log!("DEBUG: Inner error updating job {}: {:?}", id, e);
            Err(e)
        }
        Err(e) => {
            leptos::logging::log!("DEBUG: Spawn blocking error updating job {}: {:?}", id, e);
            Err(srv_err(format!("Spawn error: {}", e)))
        }
    }
}

#[cfg(feature = "ssr")]
pub async fn add_job_log(
    pool: r2d2::Pool<duckdb::DuckdbConnectionManager>,
    job_id: String,
    message: String,
) -> Result<(), ServerFnError> {
    leptos::logging::log!("DEBUG: Adding log for job {}: {}", job_id, message);
    let id = uuid::Uuid::new_v4().to_string();
    
    let res = tokio::task::spawn_blocking(move || -> Result<(), ServerFnError> {
        let conn = pool.get().map_err(srv_err)?;
        conn.execute(
            "INSERT INTO job_logs (id, job_id, message, timestamp) VALUES (?, ?, ?, CURRENT_TIMESTAMP);",
            params![id, job_id, message],
        ).map_err(srv_err)?;
        Ok(())
    }).await;

    match res {
        Ok(Ok(_)) => Ok(()),
        Ok(Err(e)) => Err(e),
        Err(e) => Err(srv_err(format!("Spawn error: {}", e))),
    }
}

#[server(ListJobLogs, "/api")]
pub async fn list_job_logs(job_id: String) -> Result<Vec<JobLog>, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        use axum::extract::Extension;
        use duckdb::DuckdbConnectionManager;
        use leptos_axum::extract;
        use r2d2::Pool;

        let Extension(pool) = extract::<Extension<Pool<DuckdbConnectionManager>>>()
            .await
            .map_err(|_| srv_err("Failed to extract DB pool"))?;

        let logs = tokio::task::spawn_blocking(move || -> Result<Vec<JobLog>, ServerFnError> {
            let conn = pool.get().map_err(srv_err)?;
            let mut stmt = conn
                .prepare("SELECT id, job_id, message, strftime(timestamp, '%Y-%m-%d %H:%M:%S') FROM job_logs WHERE job_id = ? ORDER BY timestamp ASC;")
                .map_err(srv_err)?;
            let mut rows = stmt.query(params![job_id]).map_err(srv_err)?;
            let mut logs = Vec::new();
            while let Some(row) = rows.next().map_err(srv_err)? {
                logs.push(JobLog {
                    id: row.get(0).map_err(srv_err)?,
                    job_id: row.get(1).map_err(srv_err)?,
                    message: row.get(2).map_err(srv_err)?,
                    timestamp: row.get(3).map_err(srv_err)?,
                });
            }
            Ok(logs)
        })
        .await
        .map_err(srv_err)??;

        Ok(logs)
    }
    #[cfg(not(feature = "ssr"))]
    { unreachable!() }
}

#[server(ListJobs, "/api")]
pub async fn list_jobs() -> Result<Vec<Job>, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        use axum::extract::Extension;
        use duckdb::DuckdbConnectionManager;
        use leptos_axum::extract;
        use r2d2::Pool;

        let Extension(pool) = extract::<Extension<Pool<DuckdbConnectionManager>>>()
            .await
            .map_err(|_| srv_err("Failed to extract DB pool"))?;

        let jobs = tokio::task::spawn_blocking(move || -> Result<Vec<Job>, ServerFnError> {
            let conn = pool.get().map_err(srv_err)?;
            let mut stmt = conn
                .prepare(
                    "SELECT id, name, status, progress, strftime(started_at, '%Y-%m-%d %H:%M:%S'), strftime(finished_at, '%Y-%m-%d %H:%M:%S') \
                     FROM jobs \
                     ORDER BY started_at DESC \
                     LIMIT 50;",
                )
                .map_err(srv_err)?;
            let mut rows = stmt.query(params![]).map_err(srv_err)?;
            let mut jobs = Vec::new();
            while let Some(row) = rows.next().map_err(srv_err)? {
                jobs.push(Job {
                    id: row.get::<_, String>(0).map_err(srv_err)?,
                    name: row.get::<_, String>(1).map_err(srv_err)?,
                    status: row.get::<_, String>(2).map_err(srv_err)?,
                    progress: row.get::<_, i32>(3).map_err(srv_err)?,
                    started_at: row.get::<_, String>(4).map_err(srv_err)?,
                    finished_at: row.get::<_, Option<String>>(5).map_err(srv_err)?,
                });
            }
            Ok(jobs)
        })
        .await
        .map_err(srv_err)??;

        Ok(jobs)
    }
    #[cfg(not(feature = "ssr"))]
    { unreachable!() }
}

#[server(ClearCompletedJobs, "/api")]
pub async fn clear_completed_jobs() -> Result<(), ServerFnError> {
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
            conn.execute("DELETE FROM jobs WHERE status='completed' OR status='failed';", params![]).map_err(srv_err)?;
            Ok(())
        })
        .await
        .map_err(srv_err)??;

        Ok(())
    }
    #[cfg(not(feature = "ssr"))]
    { unreachable!() }
}
