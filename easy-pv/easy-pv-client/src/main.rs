use easy_pv_lib_client::{client::start_client, config::ClientConfig};
use tokio::signal;

#[tokio::main]
async fn main() {
    println!("Hello, world!");
    let config = ClientConfig {};
    start_client(config).await;
}
