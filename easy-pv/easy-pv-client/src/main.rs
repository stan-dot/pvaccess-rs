use std::net::IpAddr;

use easy_pv_lib_client::{client::start_client, config::ClientConfig};

#[tokio::main]
async fn main() {
    println!("Dev client using hard coded client config!");
    let config = ClientConfig {
        udp_host: IpAddr::from([127, 0, 0, 1]),
        udp_port: 5576,
        tcp_port: 5576,
        buffer_size: 105576,
        introspection_registry_max_size: 15576,
    };
    // here mpsc receiver
    let (tx, mut rx) = tokio::sync::mpsc::channel(32);
    start_client(config, rx).await;
}

#[cfg(test)]
mod tests {
    use easy_pv_lib_client::helpers::spawn_test_client;
    use tokio::time::{Duration, sleep};

    #[tokio::test]
    async fn test_two_clients() {
        let c1 = tokio::spawn(spawn_test_client("Client-1", 0));
        let c2 = tokio::spawn(spawn_test_client("Client-2", 1));

        // let _ = tokio::join!(c1, c2);

        sleep(Duration::from_secs(5)).await;

        println!("🛑 Time's up, terminating test...");

        // Let tasks shut down gracefully (they’ll auto-drop)
        c1.abort();
        c2.abort();

        println!("✅ Two-client test completed.");
    }
}
