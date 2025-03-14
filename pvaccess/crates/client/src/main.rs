use rmp_serde::{decode, encode};
use serde::{Deserialize, Serialize};
use std::str;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpStream, UdpSocket};

#[derive(Serialize, Deserialize, Debug)]
struct Msg {
    msg_type: String,
    content: String,
}

#[tokio::main]
async fn main() {
    // 1️⃣ Discover TCP server via UDP beacon
    let server_addr = discover_server().await;
    println!("Connecting to TCP server at {}", server_addr);

    // 2️⃣ Connect to TCP server
    if let Ok(mut stream) = TcpStream::connect(&server_addr).await {
        println!("Connected to TCP server!");

        // Receive and decode the welcome message
        let mut buffer = vec![0; 1024];
        let n = stream.read(&mut buffer).await.unwrap();
        if let Ok(msg) = decode::from_read::<_, Msg>(&buffer[..n]) {
            println!("Received: {:?}", msg);
        }

        // Send a MessagePack-encoded message
        let my_msg = Msg {
            msg_type: "chat".to_string(),
            content: "Hello, Server!".to_string(),
        };

        let mut buf = Vec::new();
        encode::write(&mut buf, &my_msg).unwrap();
        stream.write_all(&buf).await.unwrap();
    }
}

// 🔹 Discover the TCP server via UDP broadcast
async fn discover_server() -> String {
    let socket = UdpSocket::bind("0.0.0.0:9999").await.unwrap();
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
