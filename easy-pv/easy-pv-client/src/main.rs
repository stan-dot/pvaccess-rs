use std::net::IpAddr;

use easy_pv_lib_client::{client::{start_client, start_client_v2}, config::ClientConfig};
use tokio::signal;

#[tokio::main]
async fn main() {
    println!("Hello, world!");
    let config = ClientConfig {
        udp_host: IpAddr::from([127, 0, 0, 1]),
        udp_port: 5576,
        tcp_port: 5576,
        buffer_size: 105576,
    };
    // start_client(config).await;
    start_client_v2(config).await;
}
