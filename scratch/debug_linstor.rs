
use reqwest::Client;
use serde_json::Value;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = Client::new();
    let base_url = "http://10.0.0.1:3370/v1"; // Replace with actual controller IP if I can find it

    println!("Checking Linstor Nodes...");
    let nodes: Value = client.get(&format!("{}/view/nodes", base_url)).send().await?.json().await?;
    println!("Nodes: {}", serde_json::to_string_pretty(&nodes)?);

    println!("\nChecking Storage Pools...");
    let pools: Value = client.get(&format!("{}/view/storage-pools", base_url)).send().await?.json().await?;
    println!("Pools: {}", serde_json::to_string_pretty(&pools)?);

    println!("\nChecking Resource Definitions...");
    let res_defs: Value = client.get(&format!("{}/resource-definitions", base_url)).send().await?.json().await?;
    println!("Res Defs: {}", serde_json::to_string_pretty(&res_defs)?);

    println!("\nChecking Resources (Assignments)...");
    let resources: Value = client.get(&format!("{}/view/resources", base_url)).send().await?.json().await?;
    println!("Resources: {}", serde_json::to_string_pretty(&resources)?);

    println!("\nChecking Error Reports...");
    let errors: Value = client.get(&format!("{}/error-reports", base_url)).send().await?.json().await?;
    println!("Errors: {}", serde_json::to_string_pretty(&errors)?);

    Ok(())
}
