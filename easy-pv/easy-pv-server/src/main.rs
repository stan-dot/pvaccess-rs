use easy_pv_lib_server::{add, config::AppConfig, server::start_server};

#[tokio::main]
async fn main() {
    info!("Dev server using hard coded server config!");
    let config = AppConfig::new();
    info!("Starting server with Config: {:?}", config);
    start_server(config).await;
}
