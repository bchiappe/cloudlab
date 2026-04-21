#[tokio::main]
async fn main() {
    let client = reqwest::Client::new();
    let resp = client.get("http://192.168.1.185:3370/v1/nodes/cloudlab-1/storage-pools").send().await.unwrap();
    println!("{}", resp.text().await.unwrap());
}
