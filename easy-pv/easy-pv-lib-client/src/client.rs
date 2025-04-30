use std::net::{IpAddr, SocketAddr};

use crate::config::ClientConfig;
use crate::tcp::handle_tcp_session;
use easy_pv_datatypes::{
    header::{Command, PvAccessHeader},
    messages::pv_beacon::BeaconMessage,
};

use tokio::{
    net::{TcpStream, UdpSocket},
    signal,
    sync::watch,
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

    // todo auto mode switching is not working yet
    let (mode_tx, mode_rx) = watch::channel(Mode::Tcp);
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
        mode_tx.clone(),
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
    mode_tx: watch::Sender<Mode>,
    mut mode_rx: watch::Receiver<Mode>,
    beacon_rx: watch::Receiver<BeaconMessage>,
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
                println!("✅ TCP session established.");
                if let Err(e) = handle_tcp_session(stream, &config).await {
                    println!("❌ Error during TCP session: {}", e);
                    // switch back to UDP mode
                    let _ = mode_tx.send(Mode::Udp);
                }
            }
            Err(e) => {
                println!("TCP connection failed: {}", e);
                // Could revert to UDP here if desired
                // beacon_rx.
                // mode_rx
                //     .send(Mode::Udp)
                //     .expect("Failed to switch to UDP mode");
                println!("not yet Switching to UDP mode.");
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
                if size < PvAccessHeader::LEN {
                    println!("Received packet too short for header: {} bytes", size);
                    continue;
                }

                // Step 1: Parse header first
                let header_bytes = &buf[..PvAccessHeader::LEN];
                // println!("full buffer for reference {:?}", buf);
                let header = match PvAccessHeader::from_bytes(header_bytes) {
                    Ok(h) => h,
                    Err(e) => {
                        println!("Invalid header: {}", e);
                        continue;
                    }
                };
                println!("udp header bytes are {:?}", header_bytes);
                println!("Parsed udp message header: {:?}", header);

                // Step 2: Check if the full body is present
                let expected_len = PvAccessHeader::LEN + header.payload_size as usize;
                if size < expected_len {
                    println!("Incomplete frame: expected {}, got {}", expected_len, size);
                    continue;
                }

                // Step 3: Extract body and parse if command is expected
                let body_bytes = &buf[PvAccessHeader::LEN..expected_len];

                match Command::from(header.message_command) {
                    Command::Echo => {
                        println!("Got echo over UDP (unexpected?)");
                        // usually TCP, might be misrouted
                    }
                    Command::Beacon => {
                        match BeaconMessage::from_bytes(body_bytes) {
                            Ok(beacon) => {
                                println!("Parsed beacon: {:?}", beacon);
                                let _ = beacon_tx.send(beacon.clone());
                                let _ = mode_tx.send(Mode::Tcp);
                                return; // switch to TCP mode
                            }
                            Err(e) => {
                                println!("Failed to parse beacon body: {}", e);
                            }
                        }
                    }
                    _ => {
                        println!(
                            "Received unknown command: {:?}, ignoring",
                            header.message_command
                        );
                        println!("body bytes are {:?}", body_bytes);
                    }
                }
            }
            Err(e) => eprintln!("UDP recv error: {}", e),
        }
    }
}
