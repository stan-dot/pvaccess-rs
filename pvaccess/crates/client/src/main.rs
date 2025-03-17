use config::{Config, File};
use protocol::{Msg, MsgType};
use rmp_serde::{decode, encode};
use std::collections::HashMap;
use std::env;
use std::str;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpStream, UdpSocket};

#[tokio::main]
async fn main() {
    println!("Looking for a config file...");

    // 🔹 1️⃣ Determine Config Path
    let config_path =
        env::var("CONFIG_PATH").unwrap_or_else(|_| "crates/client/config/client".to_string());

    println!("Loading config from: {}", config_path);

    // 🔹 2️⃣ Load Config
    let settings = Config::builder()
        .add_source(File::with_name(&config_path))
        .build()
        .expect("Failed to load configuration");

    let network: HashMap<String, String> = settings.get("network").unwrap();
    let udp_port: u16 = network["udp_port"].parse().unwrap();
    // let tcp_server: String = network["tcp_server"].clone();

    // 🔹 2️⃣ Discover TCP Server via UDP Beacon
    let server_addr = discover_server(udp_port).await;
    println!("Discovered TCP Server: {}", server_addr);

    // 🔹 3️⃣ Connect to the TCP Server
    if let Ok(mut stream) = TcpStream::connect(&server_addr).await {
        println!("Connected to TCP server!");

        // 🔹 4️⃣ Receive and decode the connection validation message
        let mut buffer = vec![0; 1024];
        let n = stream.read(&mut buffer).await.unwrap();
        if let Ok(msg) = decode::from_read::<_, Msg>(&buffer[..n]) {
            println!("Received validation message: {:?}", msg);
        }

        // 🔹 5️⃣ Send an Echo Message
        let echo_msg = Msg {
            msg_type: MsgType::Echo,
            content: "Hello, Server!".to_string(),
        };

        let mut buf = Vec::new();
        encode::write(&mut buf, &echo_msg).unwrap();
        stream.write_all(&buf).await.unwrap();

        // 🔹 6️⃣ Keep Connection Alive Until SIGTERM
        println!("Client is now waiting for SIGTERM...");
        wait_for_shutdown().await;
    }
}

// 🔹 Discover the TCP Server via UDP
async fn discover_server(udp_port: u16) -> String {
    let socket = UdpSocket::bind(("0.0.0.0", udp_port)).await.unwrap();
    let mut buffer = [0; 128];

    loop {
        if let Ok((size, _)) = socket.recv_from(&mut buffer).await {
            let msg = str::from_utf8(&buffer[..size]).unwrap();
            if msg.starts_with("DISCOVER_SERVER:") {
                return msg.replace("DISCOVER_SERVER:", "").trim().to_string();
            }
        }
    }
}

// 🔹 Wait for SIGTERM before exiting
async fn wait_for_shutdown() {
    tokio::signal::ctrl_c()
        .await
        .expect("Failed to listen for SIGINT");
    println!("Received SIGINT, closing connection.");
}
