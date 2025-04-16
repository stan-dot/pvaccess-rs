use config::{Config, File};
use std::env;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{TcpListener, TcpStream, UdpSocket},
    signal,
    sync::{RwLock, oneshot},
    time::{Duration, interval},
};
use uuid::Uuid;

use std::collections::HashMap;
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};

use crate::websocket::start_websocket_server;
use protocol::pvaccess::{client_manager::ClientManager, pv_beacon::BeaconMessage};
pub mod websocket;

#[tokio::main]
async fn main() {
    let server_guid = Uuid::new_v4();

    println!("Server GUID: {}", server_guid);
    println!("Looking for a config file...");
    let config_path =
        env::var("CONFIG_PATH").unwrap_or_else(|_| "crates/pv-server/config/server".to_string());
    println!("Loading config from: {}", config_path);
    let settings = Config::builder()
        .add_source(File::with_name(&config_path))
        .build()
        .expect("Failed to load pv-server configuration");
    let network_settings: HashMap<String, String> = settings.get("network").unwrap();
    let tcp_addr: String = network["tcp_addr"].clone();

    let manager = Arc::new(ClientManager::new());

    // Start WebSocket server
    let ws_manager = Arc::clone(&manager);
    let port = 8080; // todo make this configurable
    tokio::spawn(start_websocket_server(ws_manager, tcp_addr.clone(), port));

    // 🔹 2️⃣ Create a shutdown signal (Ctrl+C)
    let (shutdown_tx, shutdown_rx) = oneshot::channel::<()>();
    let udp_active = Arc::new(AtomicBool::new(true));
    let shared_settings = Arc::new(RwLock::new(network));

    // 🔹 3️⃣ Start UDP Beacon Task
    let udp_active_clone = Arc::clone(&udp_active);
    let udp_task = tokio::spawn(async move {
        send_udp_beacons(udp_active_clone, shared_settings.clone()).await;
    });

    let listener = TcpListener::bind(&tcp_addr).await.unwrap();

    println!("TCP Server running on {}", tcp_addr);
    let tcp_task = tokio::spawn(async move {
        loop {
            let (socket, addr) = loistener.acccept().await.unwrap();
            let client_manager = Arc::clone(&manager);
            // todo this line or similar one
            // tokio::spawn(handle_client(stream, addr, client_manager));
            println!("New TCP client connected: {}", addr);
            let id_string = server_guid.to_string();
            tokio::spawn(handle_tcp_client(socket, id_string, manager));
        }
    });
    let mut terminate_signal = signal::unix::signal(signal::unix::SignalKind::terminate()).unwrap();

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
    let message = BeaconMessage {
        guid: Uuid::new_v4().as_bytes()[..12].try_into().unwrap(), // Truncate to 12 bytes
        flags: 0,
        beacon_sequence_id: todo!(), // todo make this increment as the item changes
        change_count: todo!(),       // every time the list of channels changes
        server_address: todo!(),     // todo read from env var I guess
        server_port: 8000,
        protocol: "tcp".to_owned(),
        server_status_if: "test server status data field".to_owned(),
    };

    let serialized_message = message.to_bytes().unwrap();
    let mut ticker = interval(Duration::from_secs(initial_interval));
    for i in 0..15 {
        if !active.load(Ordering::Relaxed) {
            println!("UDP beacon stopped before interval switch.");
            return;
        }
        if let Err(e) = socket.send_to(&serialized_message, &beacon_addr).await {
            eprintln!("failed to send UDP beacon {:?}", e);
        } else {
            println!("send UDP beacon to {}", addr);
        }
        println!(
            "🔹 Sent UDP beacon #{} (every {}s)",
            i + 1,
            initial_interval
        );
        ticker.tick().await;
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
            if let Err(e) = socket.send_to(&serialized_message, &beacon_addr).await {
                eprintln!("failed to send UDP beacon {:?}", e);
            } else {
                println!("send UDP beacon to {}", addr);
            }
            println!(
                "🟢 Sent long-term UDP beacon (every {}s)",
                long_term_interval
            );
            long_term_ticker.tick().await;
        }
    }
}

async fn handle_tcp_client(
    mut socket: TcpStream,
    validation_extra: String,
    manager: Arc<ClientManager>,
) {
    let validation_msg = ConnectionValidationRequest {
        server_receive_buffer_size: todo!(),
        server_introspection_registry_max_size: todo!(),
        auth_nz: todo!(),
    };

    let validation_bytes = validation_msg.to_bytes().unwrap();
    let _ = socket.write_all(&validation_bytes).await;

    loop {
        match socket.read(&mut buffer).await {
            Ok(0) => {
                println!("Client disconnected");
                // todo add the client socket reference
                // manager.remove_client("some address")
                break;
            }
            Ok(n) => {
                // todo change decoding from the bytes one
                if let Ok(msg) = decode::from_read::<_, Msg>(&buffer[..n]) {
                    println!("Received: {:?}", msg);
                    // todo decode this correctly
                    let validation_response_msg =
                        ConnectionValidationResponse::from_bytes(&msg).unwrap();
                    // todo keep the connection here and enable CRUD messages over the channel pool
                }
            }
            Err(_) => break,
        }
    }
}
