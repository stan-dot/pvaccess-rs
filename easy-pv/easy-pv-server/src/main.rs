use easy_pv_lib_server::{add, config::AppConfig, register_features, server::start_server};

#[tokio::main]
async fn main() {
    let r = add(1, 2);
    println!("1 + 2 = {}", r);

    let features = register_features();
    println!("Hello, world!");
    let config = AppConfig::new();
    println!("Config: {:?}", config);
    start_server().await;
}
