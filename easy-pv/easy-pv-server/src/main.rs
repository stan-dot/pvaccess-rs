use easy_pv_lib_server::{config::AppConfig, server::start_server};
use tracing::info;
use tracing_subscriber;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .init();
    info!("Dev server using hard coded server config!");
    let config = AppConfig::new();
    info!("Starting server with Config: {:?}", config);
    start_server(config).await;
}
