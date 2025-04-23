use std::net::IpAddr;

use crate::config::ClientConfig;

use tokio::{
    net::{UdpSocket, tcp},
    signal,
    sync::oneshot,
};
// todo here add client state struct that will hold from udp beacon to tcp connection

pub async fn start_client(config: ClientConfig) {
    let mut terminate_signal = signal::unix::signal(signal::unix::SignalKind::terminate()).unwrap();
    let (shutdown_tx, shutdown_rx) = oneshot::channel::<()>();

    let udp_port = config.udp_port;
    let tcp_port = config.tcp_port;
    // todo init a udp_task using extracted port
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
    let mut buffer = [0; 128];

    loop {
        if let Ok((size, _)) = socket.recv_from(&mut buffer).await {
            // todo must parse the frames to get the BeaconMessage type
            let msg = str::from_utf8(&buffer[..size]).unwrap();
            if msg.starts_with("DISCOVER_SERVER:") {
                return msg.replace("DISCOVER_SERVER:", "").trim().to_string();
            }
        }
    }
}
