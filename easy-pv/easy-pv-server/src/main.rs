use easy_pv_lib_server::{add, config::AppConfig};

fn main() {
    let r = add(1, 2);
    println!("1 + 2 = {}", r);

    println!("Hello, world!");
    let config = AppConfig::new();
    println!("Config: {:?}", config);
}
