use client_lib::client::Client;
use tokio::task;
use tokio::{
    join,
    time::{Duration, sleep},
};

#[tokio::main]
async fn main() {
    let client = Client::new(
        Some(9999),                // UDP Port Override
        Some(8000),                // TCP Port Override
        Some(1024),                // Buffer Size Override
        Some("127.0.0.1".into()), // Server Address Override
        Some("0.0.0.0".into()),   // UDP Bind Address Override
    );
    // todo check if server address override works ok
    let another_client = Client::new(
        Some(9999),                // UDP Port Override
        Some(8000),                // TCP Port Override
        Some(1024),                // Buffer Size Override
        Some("127.0.0.2".into()), // Server Address Override
        Some("0.0.0.1".into()),   // UDP Bind Address Override
    );

    // 🔹 Run both clients concurrently
    let task1 = task::spawn(async move {
        run_client_loop(client).await;
    });

    let task2 = task::spawn(async move {
        run_client_loop(another_client).await;
    });

    // Wait for both tasks to complete (they run indefinitely)
    let _ = join!(task1, task2);
}

// 🔹 Function to manage a single client's lifecycle
async fn run_client_loop(mut client: Client) {
    loop {
        if let Some(_) = client.discover_server().await {
            match client.connect_and_listen().await {
                Ok(_) => {
                    println!("✅ Client connected and finished. Searching for a new server...")
                }
                Err(e) => eprintln!("⚠️ Client connection error: {}. Retrying discovery...", e),
            }
        }

        // Wait before retrying to avoid excessive spam
        sleep(Duration::from_secs(5)).await;
    }
}
