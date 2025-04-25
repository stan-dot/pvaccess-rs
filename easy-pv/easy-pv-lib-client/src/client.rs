use std::net::{IpAddr, SocketAddr};

use crate::config::ClientConfig;
use easy_pv_datatypes::messages::pv_beacon::BeaconMessage;

use tokio::{
    net::{TcpStream, UdpSocket, tcp},
    signal,
    sync::{oneshot, watch},
};
// todo here add client state struct that will hold from udp beacon to tcp connection

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Mode {
    Udp,
    Tcp,
}

pub async fn start_client(config: ClientConfig) {
    println!("starting the client v1 with config: {}", config);
    let mut terminate_signal = signal::unix::signal(signal::unix::SignalKind::terminate()).unwrap();
    let (shutdown_tx, shutdown_rx) = oneshot::channel::<()>();

    let udp_port = config.udp_port;
    let tcp_port = config.tcp_port;
    // todo init a udp_task using extracted port
    // only one or the other is active at the same time. it's udp until tcp is discovered
    // if tcp connection drops, revert to udp
    tokio::select! {
        _ = signal::ctrl_c() => {
            println!("Received shutdown signal, stopping client...");
        }
        _ = terminate_signal.recv() => {
            println!("Received SIGTERM (Kubernetes shutdown), stopping client...");
        }
        _ = shutdown_rx => {
            println!("Shutdown initiated...");
        }
    }

    // Perform Graceful Shutdown
    // udp_task.abort();
    // tcp_task.abort();
    println!("client shut down gracefully.");
}

async fn discover_server(udp_host: IpAddr, udp_port: u16) -> String {
    println!("trying to discover server at port {}", udp_port);
    let socket = UdpSocket::bind((udp_host, udp_port)).await.unwrap();
    let mut buffer = [0; 1024];

    loop {
        if let Ok((size, _)) = socket.recv_from(&mut buffer).await {
            // todo must parse the frames to get the BeaconMessage type
            // let msg = str::from_utf8(&buffer[..size]).unwrap();
            // if msg.starts_with("DISCOVER_SERVER:") {
            //     return msg.replace("DISCOVER_SERVER:", "").trim().to_string();
            // }
        }
    }
}

pub async fn start_client_v2(config: ClientConfig) {
    println!("starting the client v1 with config: {}", config);
    let mut terminate_signal = signal::unix::signal(signal::unix::SignalKind::terminate()).unwrap();

    let (mode_tx, mode_rx) = watch::channel(Mode::Udp);
    let (beacon_tx, beacon_rx) = watch::channel(BeaconMessage {
        guid: [0; 12],
        flags: 0,
        beacon_sequence_id: 0,
        change_count: 0,
        server_address: IpAddr::from([0, 0, 0, 0]),
        server_port: 0,
        protocol: "unknown".to_string(),
        server_status_if: 0,
    });

    let udp_task = tokio::spawn(run_udp_mode(
        config.clone(),
        mode_tx.clone(),
        beacon_tx.clone(),
    ));
    let tcp_task = tokio::spawn(run_tcp_mode(
        config.clone(),
        mode_rx.clone(),
        beacon_rx.clone(),
    ));

    tokio::select! {
        _ = signal::ctrl_c() => println!("Ctrl-C, shutting down..."),
        _ = terminate_signal.recv() => println!("SIGTERM, shutting down..."),
    }

    udp_task.abort();
    tcp_task.abort();
    println!("Client shut down gracefully.");
}

async fn run_tcp_mode(
    config: ClientConfig,
    mut mode_rx: watch::Receiver<Mode>,
    mut beacon_rx: watch::Receiver<BeaconMessage>,
) {
    loop {
        mode_rx.changed().await.ok();
        if *mode_rx.borrow() != Mode::Tcp {
            continue;
        }

        if beacon_rx.borrow().protocol != "tcp" {
            println!("Beacon protocol is not TCP, skipping connection.");
            continue;
        }

        let beacon: BeaconMessage = beacon_rx.borrow().clone();
        let server_ip = beacon.server_address;

        let server_port = beacon.server_port;

        println!(
            "Trying to connect to TCP server at {}:{}",
            server_ip, server_port
        );

        match TcpStream::connect((server_ip, server_port)).await {
            Ok(stream) => {
                println!("TCP session established.");
                // read/write as usual
            }
            Err(e) => {
                println!("TCP connection failed: {}", e);
                // Could revert to UDP here if desired
            }
        }
    }
}

async fn run_udp_mode(
    config: ClientConfig,
    mode_tx: watch::Sender<Mode>,
    beacon_tx: watch::Sender<BeaconMessage>,
) {
    let bind_addr = SocketAddr::from((config.udp_host, config.udp_port));
    let socket = UdpSocket::bind(bind_addr)
        .await
        .expect("Failed to bind UDP");

    println!("Listening for server beacons on UDP: {}", bind_addr);
    let mut buf = vec![0u8; 1500];

    loop {
        match socket.recv_from(&mut buf).await {
            Ok((size, _src)) => {
                if let Ok(beacon) = BeaconMessage::from_bytes(&buf[..size]) {
                    println!("Parsed beacon from server: {:?}", beacon);
                    beacon_tx.send(beacon.clone()).ok();
                    mode_tx.send(Mode::Tcp).unwrap();
                    return; // Let TCP mode take over
                } else {
                    println!("Received invalid beacon");
                }
            }
            Err(e) => eprintln!("UDP recv error: {}", e),
        }
    }
}
