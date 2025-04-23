use crate::{config::AppConfig, state::ServerState};
use bytes::Bytes;
use easy_pv_datatypes::codec::PvAccessDecoder;
use easy_pv_datatypes::frame::{self, PvAccessFrame};
use easy_pv_datatypes::header::{Command, PvAccessHeader};
use easy_pv_datatypes::messages::pv_echo::{EchoMessage, EchoResponse};
use easy_pv_datatypes::messages::pv_validation::{
    ConnectionQoS, ConnectionValidationRequest, ConnectionValidationResponse,
};
use futures::StreamExt;
use futures::sink::SinkExt;
use std::sync::Arc;
use tokio::{
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

            let state = Arc::new(Mutex::new(initial_server_state));

            tokio::spawn(async move {
                if let Err(e) = handle_tcp_client(socket, state, config.clone()).await {
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

async fn handle_tcp_client(
    mut stream: TcpStream,
    _state: Arc<Mutex<ServerState>>,
    config: AppConfig,
) -> Result<(), Box<dyn std::error::Error>> {
    let (reader, writer) = stream.split();
    let mut framed_read = FramedRead::new(reader, PvAccessDecoder);
    let mut framed_write = FramedWrite::new(writer, frame::PvAccessEncoder);

    // Handle the first message as a ConnectionValidationRequest
    if let Some(frame_result) = framed_read.next().await {
        let (header, payload) = frame_result?;
        if header.message_command == Command::ConnectionValidation {
            println!("Received connection validation request: {:?}", header);

            let request = ConnectionValidationRequest::from_bytes(&payload)?;
            println!("Parsed connection validation request: {:?}", request);

            // todo read out from config
            let response = ConnectionValidationResponse {
                client_receive_buffer_size: 1024,
                client_introspection_registry_max_size: 1024,
                connection_qos: ConnectionQoS::LOW_LATENCY, //todo no idea if this is correct
                auth_nz: String::new(),
            };

            let response_bytes = response.to_bytes()?;
            let response_header = PvAccessHeader::new(
                0,
                Command::ConnectionValidation,
                response_bytes.len() as u32,
            );
            let response_frame = PvAccessFrame {
                header: response_header,
                payload: Bytes::from(response_bytes),
            };

            framed_write.send(response_frame).await?;
            println!("Sent connection validation response");
        } else {
            println!("Unexpected first message: {:?}", header.message_command);
            return Err("Expected connection validation request".into());
        }
    } else {
        println!("No first message received");
        return Err("No message received from client".into());
    }

    // Continue processing other messages
    while let Some(frame_result) = framed_read.next().await {
        let (header, payload) = frame_result?;
        match header.message_command {
            Command::Ping => {
                println!("Received ping command: {:?}", header);
                let response_header = PvAccessHeader::new(0, Command::Echo, 0);
                let response_frame = PvAccessFrame {
                    header: response_header,
                    payload: Bytes::new(),
                };
                framed_write.send(response_frame).await?;
                println!("Sent ping response");
            }
            Command::Echo => {
                println!("Received echo command: {:?}", header);
                payload.iter().for_each(|b| print!("{:02X} ", b));
                let m = EchoMessage::from_bytes(&payload, header.is_big_endian())?;
                let e = EchoResponse {
                    repeated_bytes: m.random_bytes.clone(),
                };
                let response_bytes = e.to_bytes(header.is_big_endian())?;

                let response_header =
                    PvAccessHeader::new(0, Command::Echo, response_bytes.len() as u32);
                let response_frame = PvAccessFrame {
                    header: response_header,
                    payload: Bytes::from(response_bytes),
                };

                framed_write.send(response_frame).await?;
            }
            _ => {
                println!("Unhandled command: {:?}", header.message_command);
            }
        }
    }
    Ok(())
}
