use easy_pv_lib_client::{client::start_client, config::ClientConfig};
use tokio::signal;

#[tokio::main]
async fn main() {
    println!("Hello, world!");
    let config = ClientConfig {
        udp_host: 0.0.0.0,
        udp_port: 5576,
        tcp_port: 5576,
        buffer_size: 105576,
    };
    start_client(config).await;
}
