use config::{Config, File};
use std::env;
use uuid::Uuid;

use protocol::{Msg, MsgType};
use rmp_serde::{decode, encode};
use std::collections::HashMap;
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{TcpListener, TcpStream, UdpSocket},
    signal,
    sync::{RwLock, oneshot},
    time::{Duration, interval},
};
mod redis_client;

#[tokio::main]
async fn main() {
    let server_guid = Uuid::new_v4(); // Generates a new GUID on every restart
    println!("Server GUID: {}", server_guid);
    println!("Looking for a config file...");
    let config_path =
        env::var("CONFIG_PATH").unwrap_or_else(|_| "crates/server/config/server".to_string());
    println!("Loading config from: {}", config_path);
    // 🔹 1️⃣ Load Configuration
    let settings = Config::builder()
        .add_source(File::with_name(&config_path))
        .build()
        .expect("Failed to load server configuration");

    let network: HashMap<String, String> = settings.get("network").unwrap();
    let tcp_addr: String = network["tcp_addr"].clone();

    // 🔹 2️⃣ Create a shutdown signal (Ctrl+C)
    let (shutdown_tx, shutdown_rx) = oneshot::channel::<()>();
    let udp_active = Arc::new(AtomicBool::new(true));
    let shared_settings = Arc::new(RwLock::new(network));

    // 🔹 3️⃣ Start UDP Beacon Task
    let udp_active_clone = Arc::clone(&udp_active);
    let udp_task = tokio::spawn(async move {
        send_udp_beacons(udp_active_clone, shared_settings.clone()).await;
    });

    // 🔹 4️⃣ Start TCP Server
    let listener = TcpListener::bind(&tcp_addr).await.unwrap();
    println!("TCP Server running on {}", tcp_addr);

    let tcp_task = tokio::spawn(async move {
        loop {
            let (socket, addr) = listener.accept().await.unwrap();
            println!("New TCP client connected: {}", addr);

            let id_string = server_guid.to_string();
            tokio::spawn(handle_tcp_client(socket, id_string));
        }
    });
    let mut terminate_signal = signal::unix::signal(signal::unix::SignalKind::terminate()).unwrap();

    // 🔹 5️⃣ Wait for shutdown signal
    tokio::select! {
        _ = signal::ctrl_c() => {
            println!("Received shutdown signal, stopping server...");
        }
        _ = terminate_signal.recv() => {
            println!("Received SIGTERM (Kubernetes shutdown), stopping server...");
        }
        _ = shutdown_rx => {
            println!("Shutdown initiated...");
        }
    }

    // 🔹 6️⃣ Perform Graceful Shutdown
    udp_active.store(false, Ordering::Relaxed); // Stop the UDP beacon
    udp_task.await.unwrap(); // Wait for the UDP task to exit
    tcp_task.abort(); // Stop accepting new TCP clients

    println!("Server shut down gracefully.");
}

// 🔹 Handle TCP Client Connection
async fn handle_tcp_client(mut socket: TcpStream, validation_extra: String) {
    let validation_msg = Msg {
        msg_type: MsgType::ConnectionValidation,
        content: "Connection successful!".to_string() + &validation_extra.to_string(),
    };

    let mut buf = Vec::new();
    encode::write(&mut buf, &validation_msg).unwrap();
    let _ = socket.write_all(&buf).await;

    let mut buffer = vec![0; 1024];

    loop {
        match socket.read(&mut buffer).await {
            Ok(0) => {
                println!("Client disconnected");
                break;
            }
            Ok(n) => {
                if let Ok(msg) = decode::from_read::<_, Msg>(&buffer[..n]) {
                    println!("Received: {:?}", msg);

                    if let MsgType::Echo = msg.msg_type {
                        let response = Msg {
                            msg_type: MsgType::Echo,
                            content: format!("Echo: {}", msg.content),
                        };
                        let mut response_buf = Vec::new();
                        encode::write(&mut response_buf, &response).unwrap();
                        let _ = socket.write_all(&response_buf).await;
                    }
                }
            }
            Err(_) => break,
        }
    }
}

pub async fn send_udp_beacons(
    active: Arc<AtomicBool>,
    settings: Arc<RwLock<HashMap<String, String>>>,
) {
    let settings = settings.read().await;
    let beacon_addr = settings["udp_broadcast_addr"].clone();
    let initial_interval: u64 = settings["udp_initial_interval"].parse().unwrap();
    let long_term_interval: u64 = settings["udp_long_term_interval"].parse().unwrap();

    let socket = UdpSocket::bind("0.0.0.0:0").await.unwrap();
    socket.set_broadcast(true).unwrap();

    println!(
        "UDP beacon started. Initial interval: {}s, then switching to {}s.",
        initial_interval, long_term_interval
    );

    let mut ticker = interval(Duration::from_secs(initial_interval));

    for i in 0..15 {
        if !active.load(Ordering::Relaxed) {
            println!("UDP beacon stopped before interval switch.");
            return;
        }
        send_udp_message(&socket, &beacon_addr).await;
        println!(
            "🔹 Sent UDP beacon #{} (every {}s)",
            i + 1,
            initial_interval
        );
        ticker.tick().await;
    }

    println!(
        "🔄 Switching to long-term beacon interval: {}s",
        long_term_interval
    );
    let mut long_term_ticker = interval(Duration::from_secs(long_term_interval));

    loop {
        if !active.load(Ordering::Relaxed) {
            println!("UDP beacon stopped.");
            break;
        }
        send_udp_message(&socket, &beacon_addr).await;
        println!(
            "🟢 Sent long-term UDP beacon (every {}s)",
            long_term_interval
        );
        long_term_ticker.tick().await;
    }
}

async fn send_udp_message(socket: &UdpSocket, addr: &str) {
    let beacon_message = b"DISCOVER_SERVER:127.0.0.1:8000";
    if let Err(e) = socket.send_to(beacon_message, addr).await {
        eprintln!("Failed to send UDP beacon: {:?}", e);
    } else {
        println!("Sent UDP beacon to {}", addr);
    }
}
