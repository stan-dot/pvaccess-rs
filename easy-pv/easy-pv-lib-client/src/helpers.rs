use std::net::IpAddr;
use tokio::sync::mpsc;

use crate::{client::start_client, config::ClientConfig};

pub async fn spawn_test_client(name: &str, port_offset: u16) {
    let port = 5576 + port_offset;
    let config = ClientConfig {
        udp_host: IpAddr::from([127, 0, 0, 1]),
        udp_port: port,
        tcp_port: port,
        buffer_size: 105576,
        introspection_registry_max_size: 15576,
    };
    let (_tx, rx) = mpsc::channel(32);
    println!("🟢 Starting client {} on port {}", name, port);
    start_client(config, rx).await;
}
