use leptos::prelude::*;
use serde::{Serialize, Deserialize};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum UserRole {
    #[default]
    Viewer,
    Operator,
    Admin,
}

impl std::fmt::Display for UserRole {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            UserRole::Viewer => write!(f, "Viewer"),
            UserRole::Operator => write!(f, "Operator"),
            UserRole::Admin => write!(f, "Admin"),
        }
    }
}

impl From<String> for UserRole {
    fn from(s: String) -> Self {
        match s.as_str() {
            "Admin" => UserRole::Admin,
            "Operator" => UserRole::Operator,
            _ => UserRole::Viewer,
        }
    }
}

impl UserRole {
    pub fn level(&self) -> i32 {
        match self {
            UserRole::Viewer => 0,
            UserRole::Operator => 1,
            UserRole::Admin => 2,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct User {
    pub id: String,
    pub username: String,
    pub role: UserRole,
}

#[cfg(feature = "ssr")]
use duckdb::params;

#[cfg(feature = "ssr")]
pub fn srv_err(msg: impl std::fmt::Display) -> ServerFnError {
    ServerFnError::ServerError(msg.to_string())
}

#[cfg(feature = "ssr")]
pub async fn require_role(at_least: UserRole) -> Result<User, ServerFnError> {
    let user = get_user_internal().await?.ok_or_else(|| srv_err("Not authenticated"))?;
    if user.role.level() < at_least.level() {
        return Err(srv_err(format!("Permission denied: Required role level {} (you have {})", at_least, user.role)));
    }
    Ok(user)
}

#[cfg(feature = "ssr")]
thread_local! {
    static IDENTITY_CACHE: std::cell::RefCell<Option<(crate::RequestId, String, Option<User>)>> = std::cell::RefCell::new(None);
}

#[cfg(feature = "ssr")]
pub async fn get_user_internal() -> Result<Option<User>, ServerFnError> {
    use axum::extract::Extension;
    use duckdb::DuckdbConnectionManager;
    use r2d2::Pool;
    use axum::http::HeaderMap;
    use leptos_axum::extract;

    let headers = match extract::<HeaderMap>().await {
        Ok(h) => h,
        Err(_) => return Ok(None),
    };

    let request_id = extract::<Extension<crate::RequestId>>().await
        .map(|e| e.0.clone())
        .unwrap_or(crate::RequestId("none".into()));
    
    let session_cookie = headers.get("cookie")
        .and_then(|c| c.to_str().ok())
        .and_then(|c| c.split(';').find(|s| s.trim().starts_with("session_id=")))
        .map(|s| s.trim().trim_start_matches("session_id=").to_string());

    if let Some(session_id) = session_cookie {
        // Check cache
        let cached = IDENTITY_CACHE.with(|c| {
            let borrow = c.borrow();
            if let Some((rid, cid, u)) = borrow.as_ref() {
                if rid == &request_id && cid == &session_id {
                    return Some(u.clone());
                }
            }
            None
        });

        if let Some(user) = cached {
            return Ok(user);
        }

        let Extension(pool) = extract::<Extension<Pool<DuckdbConnectionManager>>>()
            .await
            .map_err(|_| srv_err("Failed to extract DB pool"))?;

        let sid_for_query = session_id.clone();
        let user = tokio::task::spawn_blocking(move || -> Result<Option<User>, ServerFnError> {
            let conn = pool.get().map_err(srv_err)?;
            let mut stmt = conn.prepare(
                "SELECT u.id, u.username, u.role FROM users u
                 JOIN sessions s ON u.id = s.user_id 
                 WHERE s.id = ? AND s.expires_at > current_timestamp::TIMESTAMP;"
            ).map_err(srv_err)?;

            let result = stmt.query_row(params![sid_for_query], |row| {
                let role_str: String = row.get(2)?;
                Ok(User {
                    id: row.get(0)?,
                    username: row.get(1)?,
                    role: UserRole::from(role_str),
                })
            });

            match result {
                Ok(u) => Ok(Some(u)),
                Err(duckdb::Error::QueryReturnedNoRows) => Ok(None),
                Err(e) => Err(srv_err(e))
            }
        }).await.map_err(srv_err)??;

        // Update cache
        let user_clone = user.clone();
        let sid_clone = session_id.clone();
        IDENTITY_CACHE.with(|c| {
            *c.borrow_mut() = Some((request_id, sid_clone, user_clone));
        });

        Ok(user)
    } else {
        Ok(None)
    }
}

#[server(Login, "/api")]
pub async fn login(username: String, password: String) -> Result<(), ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        use axum::extract::Extension;
        use duckdb::DuckdbConnectionManager;
        use r2d2::Pool;
        use leptos_axum::extract;
        use axum::http::header::SET_COOKIE;
        use leptos_axum::ResponseOptions;

        let Extension(pool) = extract::<Extension<Pool<DuckdbConnectionManager>>>()
            .await
            .map_err(srv_err)?;
            
        let pool_clone = pool.clone();
        
        let user_id = tokio::task::spawn_blocking(move || -> Result<Option<String>, ServerFnError> {
            let conn = pool_clone.get().map_err(|e| { eprintln!("[login] pool.get error: {e}"); srv_err(e) })?;
            let mut stmt = conn.prepare("SELECT id, password_hash FROM users WHERE username = ?;").map_err(|e| { eprintln!("[login] prepare error: {e}"); srv_err(e) })?;
            
            let result = stmt.query_row(params![username], |row| {
                let id: String = row.get(0)?;
                let hash: String = row.get(1)?;
                Ok((id, hash))
            });

            match result {
                Ok((id, hash)) => {
                    match bcrypt::verify(&password, &hash) {
                        Ok(true) => Ok(Some(id)),
                        Ok(false) => { eprintln!("[login] password mismatch for user"); Ok(None) },
                        Err(e) => { eprintln!("[login] bcrypt error: {e}"); Ok(None) },
                    }
                }
                Err(duckdb::Error::QueryReturnedNoRows) => { eprintln!("[login] user not found"); Ok(None) }
                Err(e) => { eprintln!("[login] query error: {e}"); Err(srv_err(e)) }
            }
        }).await.map_err(srv_err)??;

        if let Some(uid) = user_id {
            let session_id = uuid::Uuid::new_v4().to_string();
            let pool_clone = pool.clone();
            let sid_clone = session_id.clone();
            
            tokio::task::spawn_blocking(move || -> Result<(), ServerFnError> {
                let conn = pool_clone.get().map_err(srv_err)?;
                conn.execute(
                    "INSERT INTO sessions (id, user_id, expires_at) VALUES (?, ?, current_timestamp::TIMESTAMP + INTERVAL 1 DAY);",
                    params![sid_clone, uid]
                ).map_err(|e| { eprintln!("[login] session insert error: {e}"); srv_err(e) })?;
                Ok(())
            }).await.map_err(srv_err)??;

            if let Some(response) = use_context::<ResponseOptions>() {
                response.insert_header(
                    SET_COOKIE,
                    axum::http::HeaderValue::from_str(&format!("session_id={}; HttpOnly; Path=/; Max-Age=86400", session_id)).unwrap()
                );
            }
            Ok(())
        } else {
            Err(srv_err("Invalid credentials"))
        }
    }
    #[cfg(not(feature = "ssr"))]
    {
        unreachable!()
    }
}

#[server(GetUser, "/api")]
pub async fn get_user() -> Result<Option<User>, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        get_user_internal().await
    }
    #[cfg(not(feature = "ssr"))]
    {
        unreachable!()
    }
}

#[server(Logout, "/api")]
pub async fn logout() -> Result<(), ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        use axum::http::header::SET_COOKIE;
        use leptos_axum::ResponseOptions;
        use axum::http::HeaderMap;
        use leptos_axum::extract;
        use duckdb::DuckdbConnectionManager;
        use r2d2::Pool;
        use axum::extract::Extension;

        if let Ok(headers) = extract::<HeaderMap>().await {
            if let Some(session_cookie) = headers.get("cookie")
                .and_then(|c| c.to_str().ok())
                .and_then(|c| c.split(';').find(|s| s.trim().starts_with("session_id=")))
                .map(|s| s.trim().trim_start_matches("session_id=").to_string()) {
                
                if let Ok(Extension(pool)) = extract::<Extension<Pool<DuckdbConnectionManager>>>().await {
                    let _ = tokio::task::spawn_blocking(move || -> Result<(), ServerFnError> {
                        let conn = pool.get().map_err(srv_err)?;
                        conn.execute("DELETE FROM sessions WHERE id = ?;", params![session_cookie]).map_err(srv_err)?;
                        Ok(())
                    }).await;
                }
            }
        }

        if let Some(response) = use_context::<ResponseOptions>() {
            response.insert_header(
                SET_COOKIE,
                axum::http::HeaderValue::from_str("session_id=; HttpOnly; Path=/; Max-Age=0").unwrap()
            );
        }
        Ok(())
    }
    #[cfg(not(feature = "ssr"))]
    {
        unreachable!()
    }
}

#[server(ChangePassword, "/api")]
pub async fn change_password(
    current_password: String,
    new_password: String,
    confirm_password: String,
) -> Result<(), ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        use axum::extract::Extension;
        use duckdb::DuckdbConnectionManager;
        use r2d2::Pool;
        use leptos_axum::extract;
        use bcrypt::DEFAULT_COST;

        if new_password != confirm_password {
            return Err(srv_err("New passwords do not match"));
        }

        let user = get_user().await?.ok_or_else(|| srv_err("Not logged in"))?;
        let Extension(pool) = extract::<Extension<Pool<DuckdbConnectionManager>>>()
            .await
            .map_err(srv_err)?;

        tokio::task::spawn_blocking(move || -> Result<(), ServerFnError> {
            let conn = pool.get().map_err(srv_err)?;
            
            // 1. Verify current password
            let mut stmt = conn.prepare("SELECT password_hash FROM users WHERE id = ?;").map_err(srv_err)?;
            let current_hash: String = stmt.query_row(params![user.id], |row| row.get(0)).map_err(srv_err)?;
            
            if !bcrypt::verify(&current_password, &current_hash).map_err(srv_err)? {
                return Err(srv_err("Incorrect current password"));
            }

            // 2. Hash and update
            let new_hash = bcrypt::hash(&new_password, DEFAULT_COST).map_err(srv_err)?;
            conn.execute("UPDATE users SET password_hash = ? WHERE id = ?;", params![new_hash, user.id]).map_err(srv_err)?;
            
            Ok(())
        }).await.map_err(srv_err)??;

        Ok(())
    }
    #[cfg(not(feature = "ssr"))]
    {
        let _ = (current_password, new_password, confirm_password);
        unreachable!()
    }
}

#[server(ListUsers, "/api")]
pub async fn list_users() -> Result<Vec<User>, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        use axum::extract::Extension;
        use duckdb::DuckdbConnectionManager;
        use r2d2::Pool;
        use leptos_axum::extract;

        let Extension(pool) = extract::<Extension<Pool<DuckdbConnectionManager>>>()
            .await
            .map_err(srv_err)?;

        let users = tokio::task::spawn_blocking(move || -> Result<Vec<User>, ServerFnError> {
            let conn = pool.get().map_err(srv_err)?;
            let mut stmt = conn.prepare("SELECT id, username, role FROM users ORDER BY username ASC;").map_err(srv_err)?;
            let mut rows = stmt.query(params![]).map_err(srv_err)?;
            let mut items = Vec::new();
            while let Some(row) = rows.next().map_err(srv_err)? {
                let role_str: String = row.get(2).map_err(srv_err)?;
                items.push(User {
                    id: row.get(0).map_err(srv_err)?,
                    username: row.get(1).map_err(srv_err)?,
                    role: UserRole::from(role_str),
                });
            }
            Ok(items)
        }).await.map_err(srv_err)??;

        Ok(users)
    }
    #[cfg(not(feature = "ssr"))]
    { unreachable!() }
}

#[server(CreateUser, "/api")]
pub async fn create_user(username: String, password: String, role: UserRole) -> Result<(), ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        require_role(UserRole::Admin).await?;
        use axum::extract::Extension;
        use duckdb::DuckdbConnectionManager;
        use r2d2::Pool;
        use leptos_axum::extract;
        use bcrypt::DEFAULT_COST;

        let Extension(pool) = extract::<Extension<Pool<DuckdbConnectionManager>>>()
            .await
            .map_err(srv_err)?;

        tokio::task::spawn_blocking(move || -> Result<(), ServerFnError> {
            let conn = pool.get().map_err(srv_err)?;
            let id = uuid::Uuid::new_v4().to_string();
            let hash = bcrypt::hash(password, DEFAULT_COST).map_err(srv_err)?;
            let role_name = role.to_string();
            conn.execute("INSERT INTO users (id, username, password_hash, role) VALUES (?, ?, ?, ?);", params![id, username, hash, role_name]).map_err(srv_err)?;
            Ok(())
        }).await.map_err(srv_err)??;

        Ok(())
    }
    #[cfg(not(feature = "ssr"))]
    { unreachable!() }
}

#[server(DeleteUser, "/api")]
pub async fn delete_user(id: String) -> Result<(), ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        require_role(UserRole::Admin).await?;
        use axum::extract::Extension;
        use duckdb::DuckdbConnectionManager;
        use r2d2::Pool;
        use leptos_axum::extract;

        let Extension(pool) = extract::<Extension<Pool<DuckdbConnectionManager>>>()
            .await
            .map_err(srv_err)?;

        tokio::task::spawn_blocking(move || -> Result<(), ServerFnError> {
            let conn = pool.get().map_err(srv_err)?;
            
            // 1. Verify it's not the admin user
            let mut stmt = conn.prepare("SELECT username FROM users WHERE id = ?;").map_err(srv_err)?;
            let username: String = stmt.query_row(params![id], |row| row.get(0)).map_err(srv_err)?;
            
            if username == "admin" {
                return Err(srv_err("The 'admin' user cannot be deleted"));
            }

            // 2. Delete sessions and user
            conn.execute("DELETE FROM sessions WHERE user_id = ?;", params![id]).map_err(srv_err)?;
            conn.execute("DELETE FROM users WHERE id = ?;", params![id]).map_err(srv_err)?;
            
            Ok(())
        }).await.map_err(srv_err)??;

        Ok(())
    }
    #[cfg(not(feature = "ssr"))]
    { unreachable!() }
}

#[server(UpdateUserRole, "/api")]
pub async fn update_user_role(id: String, role: UserRole) -> Result<(), ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        require_role(UserRole::Admin).await?;
        
        use axum::extract::Extension;
        use duckdb::DuckdbConnectionManager;
        use r2d2::Pool;
        use leptos_axum::extract;

        let Extension(pool) = extract::<Extension<Pool<DuckdbConnectionManager>>>()
            .await
            .map_err(srv_err)?;

        tokio::task::spawn_blocking(move || -> Result<(), ServerFnError> {
            let conn = pool.get().map_err(srv_err)?;
            
            // 1. Verify it's not the admin user
            let mut stmt = conn.prepare("SELECT username FROM users WHERE id = ?;").map_err(srv_err)?;
            let username: String = stmt.query_row(params![id], |row| row.get(0)).map_err(srv_err)?;
            
            if username == "admin" {
                return Err(srv_err("The 'admin' user role cannot be changed"));
            }

            // 2. Update role
            let role_name = role.to_string();
            conn.execute("UPDATE users SET role = ? WHERE id = ?;", params![role_name, id]).map_err(srv_err)?;
            
            Ok(())
        }).await.map_err(srv_err)??;

        Ok(())
    }
    #[cfg(not(feature = "ssr"))]
    { unreachable!() }
}
