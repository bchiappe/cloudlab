use leptos::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct DnsCredential {
    pub id: String,
    pub name: String,
    pub provider: String,
    pub api_key: String, // Masked on read
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ProxyEntry {
    pub id: String,
    pub domain: String,
    pub container_id: String,
    pub container_name: String,
    pub container_port: i32,
    pub ssl_enabled: bool,
    pub ssl_status: String,
    pub force_https: bool,
    pub status: String,
    pub ssl_challenge_type: String,  // "http", "dns-digitalocean", "dns-manual"
    pub dns_provider: String,
    pub dns_credential_id: String,
    pub dns_credential_name: String, // Joined
}

#[cfg(feature = "ssr")]
use duckdb::params;

#[cfg(feature = "ssr")]
use crate::auth::srv_err;

// ─── DNS Credentials ─────────────────────────────────────────────────────────

#[server(ListDnsCredentials, "/api")]
pub async fn list_dns_credentials() -> Result<Vec<DnsCredential>, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        use axum::extract::Extension;
        use duckdb::DuckdbConnectionManager;
        use leptos_axum::extract;
        use r2d2::Pool;

        let Extension(pool) = extract::<Extension<Pool<DuckdbConnectionManager>>>()
            .await
            .map_err(|_| srv_err("Failed to extract DB pool"))?;

        let creds = tokio::task::spawn_blocking(move || -> Result<Vec<DnsCredential>, ServerFnError> {
            let conn = pool.get().map_err(srv_err)?;
            let mut stmt = conn
                .prepare("SELECT id, name, provider, api_key FROM dns_credentials ORDER BY name ASC;")
                .map_err(srv_err)?;
            let mut rows = stmt.query(params![]).map_err(srv_err)?;
            let mut creds = Vec::new();
            while let Some(row) = rows.next().map_err(srv_err)? {
                creds.push(DnsCredential {
                    id: row.get::<_, String>(0).map_err(srv_err)?,
                    name: row.get::<_, String>(1).map_err(srv_err)?,
                    provider: row.get::<_, String>(2).map_err(srv_err)?,
                    api_key: {
                        let key = row.get::<_, String>(3).map_err(srv_err)?;
                        if key.len() > 4 {
                            format!("{}…{}", &key[..4], &key[key.len()-4..])
                        } else {
                            "••••".to_string()
                        }
                    },
                });
            }
            Ok(creds)
        })
        .await
        .map_err(srv_err)??;

        Ok(creds)
    }
    #[cfg(not(feature = "ssr"))]
    { unreachable!() }
}

#[server(CreateDnsCredential, "/api")]
pub async fn create_dns_credential(
    name: String,
    provider: String,
    api_key: String,
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
                "INSERT INTO dns_credentials (id, name, provider, api_key) VALUES (?, ?, ?, ?);",
                params![id, name, provider, api_key],
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

#[server(UpdateDnsCredential, "/api")]
pub async fn update_dns_credential(
    id: String,
    name: String,
    provider: String,
    api_key: String,
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
            if api_key.contains('…') || api_key == "••••" {
                conn.execute(
                    "UPDATE dns_credentials SET name=?, provider=? WHERE id=?;",
                    params![name, provider, id],
                )
                .map_err(srv_err)?;
            } else {
                conn.execute(
                    "UPDATE dns_credentials SET name=?, provider=?, api_key=? WHERE id=?;",
                    params![name, provider, api_key, id],
                )
                .map_err(srv_err)?;
            }
            Ok(())
        })
        .await
        .map_err(srv_err)??;

        Ok(())
    }
    #[cfg(not(feature = "ssr"))]
    { unreachable!() }
}

#[server(DeleteDnsCredential, "/api")]
pub async fn delete_dns_credential(id: String) -> Result<(), ServerFnError> {
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
            // Clean up dependencies or let them stay (they'll fail issuance)
            conn.execute("DELETE FROM dns_credentials WHERE id=?;", params![id])
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

// ─── Proxies ─────────────────────────────────────────────────────────────────

#[server(ListProxies, "/api")]
pub async fn list_proxies() -> Result<Vec<ProxyEntry>, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        use axum::extract::Extension;
        use duckdb::DuckdbConnectionManager;
        use leptos_axum::extract;
        use r2d2::Pool;

        let Extension(pool) = extract::<Extension<Pool<DuckdbConnectionManager>>>()
            .await
            .map_err(|_| srv_err("Failed to extract DB pool"))?;

        let proxies = tokio::task::spawn_blocking(move || -> Result<Vec<ProxyEntry>, ServerFnError> {
            let conn = pool.get().map_err(srv_err)?;
            let mut stmt = conn
                .prepare(
                    "SELECT p.id, p.domain, p.container_id, COALESCE(c.name, 'Unknown') as container_name, \
                     p.container_port, p.ssl_enabled, p.ssl_status, p.force_https, p.status, \
                     p.ssl_challenge_type, p.dns_provider, p.dns_credential_id, COALESCE(dc.name, '') as dns_credential_name \
                     FROM proxies p \
                     LEFT JOIN containers c ON p.container_id = c.id \
                     LEFT JOIN dns_credentials dc ON p.dns_credential_id = dc.id \
                     ORDER BY p.domain ASC;",
                )
                .map_err(srv_err)?;
            let mut rows = stmt.query(params![]).map_err(srv_err)?;
            let mut proxies = Vec::new();
            while let Some(row) = rows.next().map_err(srv_err)? {
                proxies.push(ProxyEntry {
                    id: row.get::<_, String>(0).map_err(srv_err)?,
                    domain: row.get::<_, String>(1).map_err(srv_err)?,
                    container_id: row.get::<_, String>(2).map_err(srv_err).unwrap_or_default(),
                    container_name: row.get::<_, String>(3).map_err(srv_err).unwrap_or_else(|_| "Unknown".into()),
                    container_port: row.get::<_, i32>(4).map_err(srv_err)?,
                    ssl_enabled: row.get::<_, bool>(5).map_err(srv_err).unwrap_or(false),
                    ssl_status: row.get::<_, String>(6).map_err(srv_err).unwrap_or_else(|_| "none".into()),
                    force_https: row.get::<_, bool>(7).map_err(srv_err).unwrap_or(true),
                    status: row.get::<_, String>(8).map_err(srv_err).unwrap_or_else(|_| "inactive".into()),
                    ssl_challenge_type: row.get::<_, String>(9).map_err(srv_err).unwrap_or_else(|_| "http".into()),
                    dns_provider: row.get::<_, String>(10).map_err(srv_err).unwrap_or_default(),
                    dns_credential_id: row.get::<_, String>(11).map_err(srv_err).unwrap_or_default(),
                    dns_credential_name: row.get::<_, String>(12).map_err(srv_err).unwrap_or_default(),
                });
            }
            Ok(proxies)
        })
        .await
        .map_err(srv_err)??;

        Ok(proxies)
    }
    #[cfg(not(feature = "ssr"))]
    { unreachable!() }
}

#[server(CreateProxy, "/api")]
pub async fn create_proxy(
    domain: String,
    container_id: String,
    container_port: i32,
    force_https: bool,
    auto_ssl: bool,
    ssl_challenge_type: String,
    dns_provider: String,
    dns_credential_id: String,
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

        let ssl_status = if !auto_ssl {
            "none".to_string()
        } else if ssl_challenge_type == "dns-manual" {
            "pending_validation".to_string()
        } else {
            "provisioning".to_string()
        };
        let ssl_enabled = auto_ssl;

        tokio::task::spawn_blocking(move || -> Result<(), ServerFnError> {
            let conn = pool.get().map_err(srv_err)?;
            let id = uuid::Uuid::new_v4().to_string();
            conn.execute(
                "INSERT INTO proxies (id, domain, container_id, container_port, ssl_enabled, ssl_status, force_https, status, ssl_challenge_type, dns_provider, dns_credential_id) \
                 VALUES (?, ?, ?, ?, ?, ?, ?, 'active', ?, ?, ?);",
                params![id, domain, container_id, container_port, ssl_enabled, ssl_status, force_https, ssl_challenge_type, dns_provider, dns_credential_id],
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

#[server(UpdateProxy, "/api")]
pub async fn update_proxy(
    id: String,
    domain: String,
    container_id: String,
    container_port: i32,
    force_https: bool,
    ssl_challenge_type: String,
    dns_provider: String,
    dns_credential_id: String,
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
                "UPDATE proxies SET domain=?, container_id=?, container_port=?, force_https=?, ssl_challenge_type=?, dns_provider=?, dns_credential_id=? WHERE id=?;",
                params![domain, container_id, container_port, force_https, ssl_challenge_type, dns_provider, dns_credential_id, id],
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

#[server(DeleteProxy, "/api")]
pub async fn delete_proxy(id: String) -> Result<(), ServerFnError> {
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
            conn.execute("DELETE FROM proxies WHERE id=?;", params![id])
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

#[server(IssueSsl, "/api")]
pub async fn issue_ssl(id: String, challenge_type: String, dns_provider: String, dns_credential_id: String) -> Result<String, ServerFnError> {
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

        let result_status = challenge_type.clone();

        tokio::task::spawn_blocking(move || -> Result<String, ServerFnError> {
            let conn = pool.get().map_err(srv_err)?;
            
            // In a real implementation, we'd fetch the API key from dns_credentials here
            // if dns_credential_id is set.
            
            // Determine next status based on challenge type
            let new_status = match challenge_type.as_str() {
                "http" => "active",
                "dns-digitalocean" => "active",
                "dns-manual" => "pending_validation",
                _ => "provisioning"
            };
            
            conn.execute(
                "UPDATE proxies SET ssl_enabled = true, ssl_status = ?, ssl_challenge_type = ?, dns_provider = ?, dns_credential_id = ? WHERE id=?;",
                params![new_status, challenge_type, dns_provider, dns_credential_id, id],
            )
            .map_err(srv_err)?;
            
            Ok(new_status.to_string())
        })
        .await
        .map_err(srv_err)??;

        let final_status = match result_status.as_str() {
            "http" | "dns-digitalocean" => "active",
            "dns-manual" => "pending_validation",
            _ => "provisioning",
        };
        Ok(final_status.to_string())
    }
    #[cfg(not(feature = "ssr"))]
    { unreachable!() }
}

#[server(ValidateDnsChallenge, "/api")]
pub async fn validate_dns_challenge(id: String) -> Result<String, ServerFnError> {
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

        tokio::task::spawn_blocking(move || -> Result<String, ServerFnError> {
            let conn = pool.get().map_err(srv_err)?;
            conn.execute(
                "UPDATE proxies SET ssl_status = 'active' WHERE id=? AND ssl_status = 'pending_validation';",
                params![id],
            )
            .map_err(srv_err)?;
            
            Ok("active".to_string())
        })
        .await
        .map_err(srv_err)??;

        Ok("active".to_string())
    }
    #[cfg(not(feature = "ssr"))]
    { unreachable!() }
}
