use config::{Config, File};
use protocol::{Msg, MsgType};
use rmp_serde::{decode, encode};
use std::collections::HashMap;
use std::env;
use std::str;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpStream, UdpSocket};
use tokio::time::{Duration, sleep};

#[tokio::main]
async fn main() {
    println!("Looking for a config file...");

    // 🔹 1️⃣ Load Config
    let config_path =
        env::var("CONFIG_PATH").unwrap_or_else(|_| "crates/client/config/client".to_string());
    println!("Loading config from: {}", config_path);
    let settings = Config::builder()
        .add_source(File::with_name(&config_path))
        .build()
        .expect("Failed to load configuration");

    let network: HashMap<String, String> = settings.get("network").unwrap();
    let udp_port: u16 = network["udp_port"].parse().unwrap();

    // 🔹 2️⃣ Keep Discovering & Reconnecting Loop
    loop {
        let server_addr = discover_server(udp_port).await;
        println!("Discovered TCP Server: {}", server_addr);

        match connect_and_listen(server_addr).await {
            Ok(_) => println!("TCP connection ended. Looking for a new server..."),
            Err(e) => eprintln!("Connection error: {}. Retrying discovery...", e),
        }

        // 🔹 Wait before retrying (avoid excessive spam)
        sleep(Duration::from_secs(5)).await;
    }
}

// 🔹 3️⃣ Connect to TCP & Listen for SIGTERM or Disconnection
async fn connect_and_listen(server_addr: String) -> Result<(), String> {
    let mut stream = TcpStream::connect(&server_addr)
        .await
        .map_err(|e| e.to_string())?;
    println!("Connected to TCP server!");

    // 🔹 Receive connection validation message
    let mut buffer = vec![0; 1024];
    let n = stream.read(&mut buffer).await.map_err(|e| e.to_string())?;
    if let Ok(msg) = decode::from_read::<_, Msg>(&buffer[..n]) {
        println!("Received validation message: {:?}", msg);
    }

    // 🔹 Send an Echo Message
    let echo_msg = Msg {
        msg_type: MsgType::Echo,
        content: "Hello, Server!".to_string(),
    };

    let mut buf = Vec::new();
    encode::write(&mut buf, &echo_msg).unwrap();
    stream.write_all(&buf).await.map_err(|e| e.to_string())?;

    // 🔹 Keep Listening for New Messages
    loop {
        tokio::select! {
            res = stream.read(&mut buffer) => {
                match res {
                    Ok(0) => {
                        println!("Server closed the connection.");
                        return Err("Server disconnected".into());
                    }
                    Ok(n) => {
                        if let Ok(msg) = decode::from_read::<_, Msg>(&buffer[..n]) {
                            println!("Received message: {:?}", msg);
                        } else {
                            eprintln!("Failed to decode message");
                        }
                    }
                    Err(e) => {
                        return Err(format!("Read error: {}", e));
                    }
                }
            }

            _ = wait_for_shutdown() => {
                println!("Received SIGTERM, exiting...");
                return Ok(());
            }
        }
    }
}

// 🔹 Discover the TCP Server via UDP
async fn discover_server(udp_port: u16) -> String {
    println!("trying to discover server at port {}", udp_port);
    let socket = UdpSocket::bind(("255.255.255.255", udp_port))
        .await
        .unwrap();
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
