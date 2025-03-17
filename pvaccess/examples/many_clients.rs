extern crate pvaccess;
use tokio::task;
use tokio::time::{sleep, Duration};
use std::env;
use config::{Config, File};
use std::collections::HashMap;

#[tokio::main]
async fn main() {
    println!("Looking for a config file...");

    // 🔹 1️⃣ Load Config
    let config_path = env::var("CONFIG_PATH").unwrap_or_else(|_| "crates/client/config/client".to_string());
    let settings = Config::builder()
        .add_source(File::with_name(&config_path))
        .build()
        .expect("Failed to load configuration");

    let network: HashMap<String, String> = settings.get("network").unwrap();
    let udp_port: u16 = network["udp_port"].parse().unwrap();

    let mut handles = vec![];

    // 🔹 2️⃣ Spawn Multiple Clients
    for i in 0..5 {
        let server_addr = discover_server(udp_port).await;
        println!("Client {} discovered TCP Server: {}", i, server_addr);

        handles.push(task::spawn(async move {
            connect_to_server(server_addr).await;
        }));

        sleep(Duration::from_secs(1)).await; // Stagger connections
    }

    // 🔹 3️⃣ Wait for all clients to complete
    for handle in handles {
        handle.await.unwrap();
    }
}
