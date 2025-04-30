use crate::{config::AppConfig, state::ServerState};
use bytes::Bytes;
use easy_pv_datatypes::codec::PvAccessDecoder;
use easy_pv_datatypes::frame::{self, PvAccessFrame};
use easy_pv_datatypes::header::{Command, PvAccessHeader};
use easy_pv_datatypes::messages::flags::PvHeaderFlags;
use easy_pv_datatypes::messages::into::{IntoPvAccessFrame, ToBytes};
use easy_pv_datatypes::messages::pv_beacon::BeaconMessage;
use easy_pv_datatypes::messages::pv_echo::{EchoMessage, EchoResponse};
use easy_pv_datatypes::messages::pv_validation::{
    ConnectionValidationRequest, ConnectionValidationResponse,
};
use futures::SinkExt;
use futures::StreamExt;
use std::sync::Arc;
use std::sync::atomic::AtomicBool;
use tokio::net::TcpStream;
use tokio::net::UdpSocket;
use tokio::sync::Mutex;
use tokio::{net::TcpListener, signal, sync::oneshot};
use tokio_util::codec::{FramedRead, FramedWrite};
use tracing::{debug, error, info};
use uuid::Uuid;

pub async fn start_server(config: AppConfig) {
    let initial_server_state = ServerState {
        connections: todo!(),
        channels: todo!(),
        logs: todo!(),
    };

    let mut terminate_signal = signal::unix::signal(signal::unix::SignalKind::terminate()).unwrap();
    let (shutdown_tx, shutdown_rx) = oneshot::channel::<()>();

    let ip = config.network.host.to_string() + ":" + &config.network.port.to_string();
    let listener = TcpListener::bind(ip).await.unwrap();

    let tcp_task = tokio::spawn(async move {
        loop {
            let (socket, addr) = listener.accept().await.unwrap();
            debug!("New connection from: {}", addr);

            let state = Arc::new(Mutex::new(initial_server_state));

            tokio::spawn(async move {
                if let Err(e) = handle_tcp_client(socket, state, config.clone()).await {
                    error!("Client error: {}", e);
                }
            });
        }
    });

    let udp_active = Arc::new(AtomicBool::new(true));
    let udp_active_clone = Arc::clone(&udp_active);
    let udp_task = tokio::spawn(async move {
        // Placeholder for UDP task
        send_udp_beacons(udp_active_clone, config.clone()).await;
    });
    tokio::select! {
        _ = signal::ctrl_c() => {
            info!("Received shutdown signal, stopping server...");
        }
        _ = terminate_signal.recv() => {
            info!("Received SIGTERM (Kubernetes shutdown), stopping server...");
        }
        _ = shutdown_rx => {
            info!("Shutdown initiated...");
        }
    }

    // Perform Graceful Shutdown
    udp_task.abort(); // Stop accepting new UDP clients
    tcp_task.abort(); // Stop accepting new TCP clients
    info!("Server shut down gracefully.");
}

async fn handle_tcp_client(
    mut stream: TcpStream,
    _state: Arc<Mutex<ServerState>>,
    config: AppConfig,
) -> Result<(), Box<dyn std::error::Error>> {
    let (reader, writer) = stream.split();
    let mut framed_read = FramedRead::new(reader, PvAccessDecoder);
    let mut framed_write = FramedWrite::new(writer, frame::PvAccessEncoder);

    // ✅ Step 1: Send ConnectionValidationRequest
    let request = ConnectionValidationRequest::new(
        config.connection_validation.receive_buffer_size,
        config.connection_validation.introspection_registry_max_size,
        Vec::new(), // authz mechanisms
    );

    let initial_flags: PvHeaderFlags = PvHeaderFlags::SEGMENT_NONE | PvHeaderFlags::BIG_ENDIAN;
    let request_frame = request.into_frame(Command::ConnectionValidation, initial_flags.bits())?;

    framed_write.send(request_frame).await?;
    info!("✅ Sent connection validation request");

    // ✅ Step 2: Wait for ConnectionValidationResponse
    if let Some(frame_result) = framed_read.next().await {
        let (header, payload) = frame_result?;
        info!("🔁 Received frame with header: {:?}", header);

        if header.message_command != Command::ConnectionValidation {
            return Err(format!(
                "❌ Expected ConnectionValidationResponse, got {:?}",
                header.message_command
            )
            .into());
        }

        let response = ConnectionValidationResponse::from_bytes(&payload)?;
        debug!("✅ Received connection validation response: {:?}", response);

        // Proceed to message loop...
    } else {
        return Err("❌ No message received from client".into());
    }

    // ✅ Step 3: Enter message-processing loop
    while let Some(frame_result) = framed_read.next().await {
        let (header, payload) = frame_result?;

        match header.message_command {
            Command::Echo => {
                let echo = EchoMessage::from_bytes(&payload, header.is_big_endian())?;
                debug!("🔁 Received Echo: {:?}", echo);

                let response = EchoResponse {
                    repeated_bytes: echo.random_bytes.clone(),
                };

                let response_bytes = response.to_bytes(header.is_big_endian())?;
                let response_header =
                    PvAccessHeader::new(0, Command::Echo, response_bytes.len() as u32);

                let response_frame = PvAccessFrame {
                    header: response_header,
                    payload: Bytes::from(response_bytes),
                };

                framed_write.send(response_frame).await?;
            }
            other => {
                warn!("⚠️ Unhandled message command: {:?}", other);
            }
        }
    }

    warn!("🔌 Client disconnected");
    Ok(())
}

async fn send_udp_beacons(udp_active: Arc<AtomicBool>, config: AppConfig) {
    let long_term_interval = config.beacon.udp_long_term_interval;
    let beacon_addr = config.network.host;
    let intitial_interval = config.beacon.udp_initial_interval;

    let socket = UdpSocket::bind((
        config.beacon.udp_server_config.host,
        config.beacon.udp_server_config.port,
    ))
    .await
    .unwrap();
    socket.set_broadcast(true).unwrap();
    info!(
        "UDP beacon started. Initial interval: {}s, then switching to {}s.",
        intitial_interval, long_term_interval
    );
    let server_guid = Uuid::new_v4().as_bytes()[..12].try_into().unwrap();

    let beacon_flags =
        PvHeaderFlags::SEGMENT_NONE | PvHeaderFlags::BIG_ENDIAN | PvHeaderFlags::FROM_SERVER;
    let message_base = BeaconMessage {
        guid: server_guid,
        flags: beacon_flags.bits(),
        beacon_sequence_id: 0,
        change_count: 0, // every time the list of channels changes
        server_address: beacon_addr,
        server_port: config.network.port,
        protocol: "tcp".to_owned(),
        server_status_if: 0, // Replace with an appropriate u8 value
    };

    let full_address = format!("{}:{}", beacon_addr, config.network.port);
    let length = 15;
    info!(
        "Short initial end UDP beacons with initial interval {}s",
        intitial_interval,
    );
    for i in 1..length {
        let mut new_messsage = message_base.clone();
        new_messsage.beacon_sequence_id = i;
        let beacon_bytes = new_messsage.into_frame(Command::Beacon, 000111).unwrap();
        let header_bytes = beacon_bytes.header.to_bytes().unwrap();
        let full_bytes = [header_bytes, beacon_bytes.payload.to_vec()].concat();

        if let Err(e) = socket.send_to(&full_bytes, &full_address).await {
            error!("failed to send UDP beacon {:?}", e);
        } else {
            info!("Short initial UDP beacon sent to {}", beacon_addr);
            info!("message, {}", message_base);
        }
        tokio::time::sleep(tokio::time::Duration::from_secs(intitial_interval)).await;
    }
    info!(
        "Switched to long term UDP beacons with interval {}s",
        long_term_interval
    );

    loop {
        let mut new_mesage = message_base.clone();
        new_mesage.beacon_sequence_id += 1;
        let beacon_frame = new_mesage.into_frame(Command::Beacon, 000111).unwrap();
        let header_bytes = beacon_frame.header.to_bytes().unwrap();
        let full_bytes = [header_bytes, beacon_frame.payload.to_vec()].concat();

        info!("Sending UDP beacon to {}", beacon_addr);
        tokio::time::sleep(tokio::time::Duration::from_secs(long_term_interval)).await;
        if let Err(e) = socket.send_to(&full_bytes, &full_address).await {
            error!("failed to send UDP beacon {:?}", e);
        } else {
            info!("Successfully sent UDP beacon to {}", beacon_addr);
        }
    }
}
