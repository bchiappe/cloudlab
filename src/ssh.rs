#[cfg(feature = "ssr")]
use crate::auth::srv_err;
use serde::{Deserialize, Serialize};
#[cfg(feature = "ssr")]
use leptos::prelude::ServerFnError;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SshKeyPair {
    pub public_key: String,
    pub private_key: String,
}

#[cfg(feature = "ssr")]
pub async fn get_cloudlab_ssh_key(
    pool: r2d2::Pool<duckdb::DuckdbConnectionManager>,
) -> Result<SshKeyPair, ServerFnError> {
    use duckdb::params;

    tokio::task::spawn_blocking(move || -> Result<SshKeyPair, ServerFnError> {
        let conn = pool.get().map_err(srv_err)?;
        
        // Try to fetch existing key
        let mut stmt = conn.prepare("SELECT value FROM global_settings WHERE key = 'ssh_public_key';").map_err(srv_err)?;
        let pub_key: Option<String> = stmt.query_row(params![], |row| row.get(0)).ok();
        
        let mut stmt = conn.prepare("SELECT value FROM global_settings WHERE key = 'ssh_private_key';").map_err(srv_err)?;
        let priv_key: Option<String> = stmt.query_row(params![], |row| row.get(0)).ok();

        if let (Some(pubk), Some(privk)) = (pub_key, priv_key) {
            return Ok(SshKeyPair { public_key: pubk, private_key: privk });
        }

        // Generate new Ed25519 key pair if not exists
        // ssh2 doesn't have a direct "generate" but we can use an external command or a different crate
        // For simplicity, let's use `ssh-keygen -t ed25519 -N "" -f /tmp/id_ed25519` then read and delete
        let tmp_file = format!("/tmp/cloudlab_id_ed25519_{}", uuid::Uuid::new_v4());
        let _ = std::process::Command::new("ssh-keygen")
            .args(&["-t", "ed25519", "-N", "", "-f", &tmp_file])
            .output()
            .map_err(srv_err)?;

        let privk = std::fs::read_to_string(&tmp_file).map_err(srv_err)?;
        let pubk = std::fs::read_to_string(format!("{}.pub", tmp_file)).map_err(srv_err)?;

        // Cleanup
        let _ = std::fs::remove_file(&tmp_file);
        let _ = std::fs::remove_file(format!("{}.pub", tmp_file));

        // Save to DB
        conn.execute("INSERT OR REPLACE INTO global_settings (key, value) VALUES ('ssh_public_key', ?);", params![pubk]).map_err(srv_err)?;
        conn.execute("INSERT OR REPLACE INTO global_settings (key, value) VALUES ('ssh_private_key', ?);", params![privk]).map_err(srv_err)?;

        Ok(SshKeyPair { public_key: pubk, private_key: privk })
    }).await.map_err(srv_err)?
}

#[cfg(feature = "ssr")]
pub fn extract_public_key(priv_key: &str) -> Result<String, ServerFnError> {
    use std::io::Write;
    
    let tmp_file = format!("/tmp/cloudlab_extract_{}", uuid::Uuid::new_v4());
    let mut file = std::fs::File::create(&tmp_file).map_err(srv_err)?;
    file.write_all(priv_key.as_bytes()).map_err(srv_err)?;
    
    // Set permissions to 600 or ssh-keygen might complain
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&tmp_file, std::fs::Permissions::from_mode(0o600)).map_err(srv_err)?;
    }

    let output = std::process::Command::new("ssh-keygen")
        .args(&["-y", "-P", "", "-f", &tmp_file])
        .output()
        .map_err(srv_err)?;

    let _ = std::fs::remove_file(&tmp_file);

    if !output.status.success() {
        let err = String::from_utf8_lossy(&output.stderr);
        return Err(srv_err(format!("Failed to extract public key: {}", err)));
    }

    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

#[cfg(feature = "ssr")]
pub fn run_remote_script(
    sess: &ssh2::Session,
    script: &str,
    password: Option<&str>,
) -> Result<(i32, String, String), ServerFnError> {
    use std::io::{Read, Write};
    
    // 1. Check if we are root
    let mut ch = sess.channel_session().map_err(srv_err)?;
    ch.exec("id -u").map_err(srv_err)?;
    let mut root_out = String::new();
    ch.read_to_string(&mut root_out).ok();
    let is_root = root_out.trim() == "0";
    let _ = ch.wait_close();

    // 2. Prepare command
    // We use bash -c to run the multi-line script. 
    // We escape single quotes inside the script.
    let escaped_script = script.replace("'", "'\\''");
    let (final_cmd, use_sudo_s) = if is_root {
        (format!("bash -c '{}'", escaped_script), false)
    } else if let Some(_) = password {
        (format!("sudo -S -p '' bash -c '{}'", escaped_script), true)
    } else {
        // No password provided, try running anyway (might have password-less sudo)
        (format!("sudo bash -c '{}'", escaped_script), false)
    };

    // 3. Execute
    let mut channel = sess.channel_session().map_err(srv_err)?;
    channel.exec(&final_cmd).map_err(srv_err)?;
    
    if use_sudo_s {
        if let Some(pw) = password {
            let _ = channel.write_all(format!("{}\n", pw).as_bytes());
            let _ = channel.flush();
        }
    }

    // 4. Read output
    let mut stdout = String::new();
    let mut stderr = String::new();
    
    channel.read_to_string(&mut stdout).map_err(srv_err)?;
    channel.stderr().read_to_string(&mut stderr).map_err(srv_err)?;
    
    let _ = channel.wait_close();
    let exit_status = channel.exit_status().map_err(srv_err)?;
    
    Ok((exit_status, stdout, stderr))
}
