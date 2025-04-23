use crate::{config::AppConfig, state::ServerState};
use easy_pv_datatypes::header::{Command, PvAccessHeader};
use easy_pv_datatypes::messages::pv_echo::{EchoMessage, EchoResponse};
use std::{collections::HashMap, net::SocketAddr, sync::Arc};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{TcpListener, TcpStream},
    signal,
    sync::{Mutex, oneshot},
};

pub async fn start_server(config: AppConfig) {
    let initial_server_state = ServerState {};

    let mut terminate_signal = signal::unix::signal(signal::unix::SignalKind::terminate()).unwrap();
    let (shutdown_tx, shutdown_rx) = oneshot::channel::<()>();

    let ip = config.network.host.to_string() + ":" + &config.network.port.to_string();
    let listener = TcpListener::bind(ip).await.unwrap();

    let tcp_task = tokio::spawn(async move {
        loop {
            let (socket, addr) = listener.accept().await.unwrap();
            println!("New connection from: {}", addr);

            // todo cloning this is not easy
            let state = Arc::new(Mutex::new(initial_server_state.clone()));
            // let state = Arc::new(Mutex::new(ServerState::new()));

            tokio::spawn(async move {
                if let Err(e) = handle_tcp_client(socket, state).await {
                    eprintln!("Client error: {}", e);
                }
            });
        }
    });

    let udp_task = tokio::spawn(async move {
        // Placeholder for UDP task
        // send_udp_beacons(udp_active_clone, shared_settings.clone()).await;
    });
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

    // Perform Graceful Shutdown
    udp_task.abort(); // Stop accepting new UDP clients
    tcp_task.abort(); // Stop accepting new TCP clients
    println!("Server shut down gracefully.");
}

async fn tcp_server_loop(
    addr: SocketAddr,
    state: Arc<Mutex<ServerState>>,
) -> tokio::io::Result<()> {
    let listener = TcpListener::bind(addr).await?;
    println!("Listening on {}", addr);

    loop {
        let (socket, addr) = listener.accept().await?;
        println!("New connection from: {}", addr);

        let state = Arc::clone(&state);

        tokio::spawn(async move {
            if let Err(e) = handle_tcp_client(socket, state).await {
                eprintln!("Client error: {}", e);
            }
        });
    }
}

async fn handle_tcp_client(
    mut stream: TcpStream,
    state: Arc<Mutex<ServerState>>,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut buffer = vec![0; 4096];
    let (mut reader, mut writer) = stream.split();

    loop {
        let n = reader.read(&mut buffer).await?;
        if n == 0 {
            println!("Connection closed");
            return Ok(());
        }

        const HEADER_LENGTH: usize = 8;
        let header = PvAccessHeader::from_bytes(&buffer[..HEADER_LENGTH])?;
        let use_big = header.is_big_endian();
        println!("Received header: {:?}", header);
        let payload_size = header.payload_size as usize;

        if payload_size + HEADER_LENGTH > buffer.len() {
            buffer.resize(payload_size + HEADER_LENGTH, 0);
        }

        let body = &buffer[HEADER_LENGTH..HEADER_LENGTH + payload_size];
        match Command::from(header.message_command) {
            Command::Ping => {
                println!("Received ping command: {:?}", header);
                // NOTE : This is a placeholder for the actual ping response
            }
            Command::Echo => {
                println!("Received echo command: {:?}", header);
                let m = EchoMessage::from_bytes(body, use_big)?;
                let e = EchoResponse {
                    repeated_bytes: m.random_bytes.clone(),
                };
                println!("Received body: {:?}", body);
                let response = e.to_bytes(use_big)?;
                writer.write_all(&response).await?;
            }
            _ => (),
        }
    }
}
