
#[cfg(feature = "ssr")]
#[tokio::main]
async fn main() {
    use axum::{Router, extract::Extension};
    use duckdb::DuckdbConnectionManager;
    use r2d2::Pool;
    use leptos::logging::log;
    use leptos::prelude::*;
    use leptos_axum::{generate_route_list, LeptosRoutes};
    use cloudlab::app::*;
    use cloudlab::storage_ssr::srv::{upload_handler, download_handler};

    let conf = get_configuration(None).unwrap();
    let addr = conf.leptos_options.site_addr;
    let leptos_options = conf.leptos_options;

    // Initialize duckdb pool
    let manager = DuckdbConnectionManager::file(".cloudlab.duckdb").unwrap();
    let pool = Pool::builder()
        .max_size(20)
        .connection_timeout(std::time::Duration::from_secs(2))
        .build(manager)
        .expect("Failed to build DB pool");

    // Initialize DuckDB tables and seed data
    {
        let conn = pool.get().unwrap();
        
        // Ensure tables exist
        conn.execute("CREATE TABLE IF NOT EXISTS users (id VARCHAR PRIMARY KEY, username VARCHAR UNIQUE, password_hash VARCHAR, role VARCHAR DEFAULT 'Viewer');", duckdb::params![]).unwrap();
        // Migration: Ensure role column exists if table already existed without it
        let _ = conn.execute("ALTER TABLE users ADD COLUMN role VARCHAR DEFAULT 'Viewer';", duckdb::params![]);
        
        conn.execute("CREATE TABLE IF NOT EXISTS sessions (id VARCHAR PRIMARY KEY, user_id VARCHAR, expires_at TIMESTAMP);", duckdb::params![]).unwrap();
        conn.execute("CREATE TABLE IF NOT EXISTS hosts (id VARCHAR PRIMARY KEY, name VARCHAR NOT NULL, address VARCHAR NOT NULL, port INTEGER NOT NULL DEFAULT 22, username VARCHAR NOT NULL DEFAULT 'root', auth_method VARCHAR NOT NULL DEFAULT 'password', password VARCHAR, ssh_key TEXT, ssh_public_key TEXT, ssh_passphrase TEXT, notes VARCHAR NOT NULL DEFAULT '', status VARCHAR NOT NULL DEFAULT 'unknown', zfs_pool_size_gb INTEGER DEFAULT 100);", duckdb::params![]).unwrap();
        // Migration: add extra columns if table already existed
        let _ = conn.execute("ALTER TABLE hosts ADD COLUMN ssh_public_key TEXT;", duckdb::params![]);
        let _ = conn.execute("ALTER TABLE hosts ADD COLUMN ssh_passphrase TEXT;", duckdb::params![]);
        let _ = conn.execute("ALTER TABLE hosts ADD COLUMN zfs_pool_size_gb INTEGER DEFAULT 100;", duckdb::params![]);
        let _ = conn.execute("ALTER TABLE hosts ADD COLUMN storage_device VARCHAR;", duckdb::params![]);
        
        conn.execute("CREATE TABLE IF NOT EXISTS vms (id VARCHAR PRIMARY KEY, host_id VARCHAR, name VARCHAR NOT NULL, cpu INTEGER DEFAULT 1, memory_mb INTEGER DEFAULT 1024, disk_size_gb INTEGER DEFAULT 20, status VARCHAR DEFAULT 'stopped', os_type VARCHAR DEFAULT 'linux', disk_volume_id VARCHAR, iso_volume_id VARCHAR, boot_device VARCHAR DEFAULT 'disk', mac_address VARCHAR, vnc_port INTEGER, vnc_token VARCHAR);", duckdb::params![]).unwrap();
        // Migration: add extra columns if table already existed
        let _ = conn.execute("ALTER TABLE vms ADD COLUMN disk_volume_id VARCHAR;", duckdb::params![]);
        let _ = conn.execute("ALTER TABLE vms ADD COLUMN iso_volume_id VARCHAR;", duckdb::params![]);
        let _ = conn.execute("ALTER TABLE vms ADD COLUMN boot_device VARCHAR DEFAULT 'disk';", duckdb::params![]);
        let _ = conn.execute("ALTER TABLE vms ADD COLUMN mac_address VARCHAR;", duckdb::params![]);
        let _ = conn.execute("ALTER TABLE vms ADD COLUMN vnc_port INTEGER;", duckdb::params![]);
        let _ = conn.execute("ALTER TABLE vms ADD COLUMN vnc_token VARCHAR;", duckdb::params![]);
        let _ = conn.execute("ALTER TABLE vms ADD COLUMN disk_size_gb INTEGER DEFAULT 20;", duckdb::params![]);

        conn.execute("CREATE TABLE IF NOT EXISTS api_keys (id VARCHAR PRIMARY KEY, label VARCHAR NOT NULL, key_prefix VARCHAR NOT NULL, hashed_key VARCHAR NOT NULL, created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP);", duckdb::params![]).unwrap();
        conn.execute("CREATE TABLE IF NOT EXISTS llms (id VARCHAR PRIMARY KEY, host_id VARCHAR, name VARCHAR NOT NULL, provider VARCHAR NOT NULL, model_name VARCHAR NOT NULL, status VARCHAR DEFAULT 'offline', repo_id VARCHAR, size_bytes BIGINT DEFAULT 0, download_status VARCHAR DEFAULT 'none', last_synced_at TIMESTAMP, UNIQUE(host_id, provider, model_name));", duckdb::params![]).unwrap();
        // Migration: allow multiple models per host+provider if we were previously restricted
        let _ = conn.execute("DROP INDEX IF EXISTS idx_llms_host_provider;", duckdb::params![]);
        // Migration: add extra columns to llms if table already existed
        let _ = conn.execute("ALTER TABLE llms ADD COLUMN repo_id VARCHAR;", duckdb::params![]);
        let _ = conn.execute("ALTER TABLE llms ADD COLUMN size_bytes BIGINT DEFAULT 0;", duckdb::params![]);
        let _ = conn.execute("ALTER TABLE llms ADD COLUMN download_status VARCHAR DEFAULT 'none';", duckdb::params![]);
        let _ = conn.execute("ALTER TABLE llms ADD COLUMN last_synced_at TIMESTAMP;", duckdb::params![]);
        // Migration: create updated unique index for multi-model ON CONFLICT support
        let _ = conn.execute("CREATE UNIQUE INDEX IF NOT EXISTS idx_llms_host_provider_model ON llms(host_id, provider, model_name);", duckdb::params![]);
        
        conn.execute("CREATE TABLE IF NOT EXISTS dns_credentials (id VARCHAR PRIMARY KEY, name VARCHAR NOT NULL, provider VARCHAR NOT NULL, api_key VARCHAR NOT NULL);", duckdb::params![]).unwrap();
        conn.execute("CREATE TABLE IF NOT EXISTS jobs (id VARCHAR PRIMARY KEY, name VARCHAR NOT NULL, status VARCHAR NOT NULL, progress INTEGER DEFAULT 0, started_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP, finished_at TIMESTAMP);", duckdb::params![]).unwrap();
        conn.execute("CREATE TABLE IF NOT EXISTS job_logs (id VARCHAR PRIMARY KEY, job_id VARCHAR NOT NULL, message TEXT NOT NULL, timestamp TIMESTAMP DEFAULT CURRENT_TIMESTAMP);", duckdb::params![]).unwrap();
        
        conn.execute("CREATE TABLE IF NOT EXISTS proxies (id VARCHAR PRIMARY KEY, domain VARCHAR NOT NULL, container_id VARCHAR, container_port INTEGER NOT NULL, ssl_enabled BOOLEAN DEFAULT false, ssl_status VARCHAR DEFAULT 'none', force_https BOOLEAN DEFAULT true, status VARCHAR DEFAULT 'inactive', created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP, ssl_challenge_type VARCHAR DEFAULT 'http', dns_provider VARCHAR DEFAULT '', dns_credential_id VARCHAR DEFAULT '');", duckdb::params![]).unwrap();
        // Migration: add challenge config columns if table already existed
        let _ = conn.execute("ALTER TABLE proxies ADD COLUMN ssl_challenge_type VARCHAR DEFAULT 'http';", duckdb::params![]);
        let _ = conn.execute("ALTER TABLE proxies ADD COLUMN dns_provider VARCHAR DEFAULT '';", duckdb::params![]);
        let _ = conn.execute("ALTER TABLE proxies ADD COLUMN dns_credential_id VARCHAR DEFAULT '';", duckdb::params![]);
        // Migration: drop old dns_api_key if it exists (replaced by dns_credential_id)
        let _ = conn.execute("ALTER TABLE proxies DROP COLUMN dns_api_key;", duckdb::params![]);

        // Chat Persistence
        conn.execute("CREATE TABLE IF NOT EXISTS chat_threads (id VARCHAR PRIMARY KEY, llm_id VARCHAR NOT NULL, title VARCHAR NOT NULL, created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP, updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP);", duckdb::params![]).unwrap();
        conn.execute("CREATE TABLE IF NOT EXISTS chat_messages (id VARCHAR PRIMARY KEY, thread_id VARCHAR NOT NULL, role VARCHAR NOT NULL, content TEXT NOT NULL, created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP);", duckdb::params![]).unwrap();

        conn.execute("CREATE TABLE IF NOT EXISTS global_settings (key VARCHAR PRIMARY KEY, value VARCHAR NOT NULL);", duckdb::params![]).unwrap();

        // Seed a default admin user if the database is empty
        let mut stmt = conn.prepare("SELECT COUNT(*) FROM users;").unwrap();
        let count: i32 = stmt.query_row(duckdb::params![], |row: &duckdb::Row| row.get(0)).unwrap_or(0);
        if count == 0 {
            use bcrypt::DEFAULT_COST;
            let hash = bcrypt::hash("password", DEFAULT_COST).unwrap();
            let id = uuid::Uuid::new_v4().to_string();
            conn.execute(
                "INSERT INTO users (id, username, password_hash, role) VALUES (?, ?, ?, ?);", 
                duckdb::params![id, "admin".to_string(), hash, "Admin".to_string()]
            ).unwrap();
        } else {
            // Ensure existing admin has Admin role
            let _ = conn.execute("UPDATE users SET role = 'Admin' WHERE username = 'admin';", duckdb::params![]);
        }
        
        // Seed default settings
        let mut stmt = conn.prepare("SELECT COUNT(*) FROM global_settings;").unwrap();
        let count: i32 = stmt.query_row(duckdb::params![], |row| row.get::<_, i32>(0)).unwrap_or(0);
        if count == 0 {
            conn.execute("INSERT INTO global_settings (key, value) VALUES ('ha_enabled', 'false');", duckdb::params![]).unwrap();
            conn.execute("INSERT INTO global_settings (key, value) VALUES ('sync_interval', '30');", duckdb::params![]).unwrap();
            conn.execute("INSERT INTO global_settings (key, value) VALUES ('cluster_name', 'Cloudlab-Cluster');", duckdb::params![]).unwrap();
            conn.execute("INSERT INTO global_settings (key, value) VALUES ('secondary_node_ip', '');", duckdb::params![]).unwrap();
        }
    }

    // Start background host monitoring
    cloudlab::hosts::start_background_monitor(pool.clone());

    // Generate the list of routes in your Leptos App
    let routes = generate_route_list(App);
    

    let app = Router::new()
        .route("/api/storage/upload", axum::routing::post(upload_handler))
        .route("/api/storage/download", axum::routing::get(download_handler))
        .route("/api/ai/v1/{*path}", axum::routing::post(cloudlab::ai_proxy::srv::ai_proxy_handler))
        .leptos_routes(&leptos_options, routes, {
            let leptos_options = leptos_options.clone();
            move || shell(leptos_options.clone())
        })
        .fallback(leptos_axum::file_and_error_handler(shell))
        .layer(axum::extract::DefaultBodyLimit::disable())
        .layer(Extension(pool))
        .with_state(leptos_options);

    // run our app with hyper
    // `axum::Server` is a re-export of `hyper::Server`
    log!("listening on http://{}", &addr);
    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    axum::serve(listener, app.into_make_service())
        .await
        .unwrap();
}

#[cfg(not(feature = "ssr"))]
pub fn main() {
    // no client-side main function
    // unless we want this to work with e.g., Trunk for pure client-side testing
    // see lib.rs for hydration function instead
}
