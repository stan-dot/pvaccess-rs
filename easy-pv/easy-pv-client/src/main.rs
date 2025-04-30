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
