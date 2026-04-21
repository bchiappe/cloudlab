use duckdb::{Connection, Result};
use std::collections::HashMap;
use reqwest::Client;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let conn = Connection::open(".cloudlab.duckdb")?;
    let mut stmt = conn.prepare("SELECT name, address FROM hosts")?;
    
    let mut rows = stmt.query([])?;
    let client = Client::builder().timeout(std::time::Duration::from_secs(3)).build()?;

    while let Some(row) = rows.next()? {
        let name: String = row.get(0)?;
        let addr: String = row.get(1)?;
        println!("Host: {} (IP: {})", name, addr);

        let url = format!("http://{}:3370/v1/view/storage-pools", addr);
        match client.get(&url).send().await {
            Ok(resp) => {
                if resp.status().is_success() {
                    if let Ok(js) = resp.json::<serde_json::Value>().await {
                        println!("Storage Pools Response: {}", serde_json::to_string_pretty(&js)?);
                    }
                } else {
                    println!("Storage Pools API returning {}", resp.status());
                    // Fallback test
                    let f_url = format!("http://{}:3370/v1/nodes/{}/storage-pools", addr, name);
                    if let Ok(f_resp) = client.get(&f_url).send().await {
                        if f_resp.status().is_success() {
                            if let Ok(f_js) = f_resp.json::<serde_json::Value>().await {
                                println!("Fallback Storage Pools Response (Linstor 1.x): {}", serde_json::to_string_pretty(&f_js)?);
                            }
                        }
                    }
                }
            }
            Err(e) => println!("Error reaching {}: {}", url, e),
        }
    }
    
    Ok(())
}
