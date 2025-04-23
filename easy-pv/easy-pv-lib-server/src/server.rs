use crate::{config::AppConfig, state::ServerState};
use easy_pv_datatypes::codec::PvAccessDecoder;
use easy_pv_datatypes::frame::{self, PvAccessFrame};
use easy_pv_datatypes::header::{Command, PvAccessHeader};
use easy_pv_datatypes::messages::pv_echo::{EchoMessage, EchoResponse};
use std::{collections::HashMap, net::SocketAddr, sync::Arc};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{TcpListener, TcpStream},
    signal,
    sync::{Mutex, oneshot},
};
use tokio_util::codec::{FramedRead, FramedWrite};

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
    let mut framed_read = FramedRead::new(reader, PvAccessDecoder);
    let mut framed_write = FramedWrite::new(writer, frame::PvAccessEncoder);

    while let Some(frame_result) = framed_read.next().await {
        let (header, payload) = frame_result?;
        match header.message_command {
            Command::Ping => {
                println!("Received ping command: {:?}", header);
                

                let response_header = PvAccessHeader::new(flags,Command::Echo, payload_size);
                let response_frame = PvAccessFrame {
                    header: PvAccessHeader {
                        magic: 0xCA,
                        version: (),
                        flags: (),
                        message_command: (),
                        payload_size: (),
                    },
                    payload: Bytes::new(),
                };
                framed_write.send(response_frame).await?;
                println!("Sent ping response");

                // NOTE : This is a placeholder for the actual ping response
            }
            Command::Echo => {
                println!("Received echo command: {:?}", header);
                let m = EchoMessage::from_bytes(payload, header.is_big_endian())?;
                let e = EchoResponse {
                    repeated_bytes: m.random_bytes.clone(),
                };
                let response_bytes = e.to_bytes(header.is_big_endian())?;

                let response_frame = PvAccessFrame {
                    header: PvAccessHeader {
                        magic: 0xCA,
                        version: header.version,
                        flags: header.flags,
                        message_command: Command::Echo,
                        payload_size: response_bytes.len() as u32,
                    },
                    payload: Bytes::from(response_bytes),
                };

                framed_write.send(response_frame).await?;
            }
            _ => {
                println!("Unhandled command: 0x{:02X}", header.message_command);
            }
        }
    }
    Ok(())
}
