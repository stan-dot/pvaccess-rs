use crate::state::ServerState;
use easy_pv_datatypes::header::PvAccessHeader;
use std::{net::SocketAddr, sync::Arc};
use tokio::{
    net::{TcpListener, TcpStream},
    signal,
    sync::{Mutex, oneshot},
};

pub async fn start_server() {
    let mut terminate_signal = signal::unix::signal(signal::unix::SignalKind::terminate()).unwrap();
    let (shutdown_tx, shutdown_rx) = oneshot::channel::<()>();


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
    println!("Server shut down gracefully.");
}
async fn tcp_server_loop(
    addr: SocketAddr,
    // features: Arc<Vec<Box<dyn Feature>>>,
    state: Arc<Mutex<ServerState>>,
) -> tokio::io::Result<()> {
    let listener = TcpListener::bind(addr).await?;
    println!("Listening on {}", addr);

    loop {
        let (socket, addr) = listener.accept().await?;
        println!("New connection from: {}", addr);

        let features = Arc::clone(&features);
        let state = Arc::clone(&state);

        tokio::spawn(async move {
            // if let Err(e) = handle_tcp_client(socket, features, state).await {
            if let Err(e) = handle_tcp_client(socket, state).await {
                eprintln!("Client error: {}", e);
            }
        });
    }
}

async fn handle_tcp_client(
    mut socket: TcpStream,
    // features: Arc<Vec<Box<dyn Feature>>>,
    state: Arc<Mutex<ServerState>>,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut buffer = vec![0; 4096];

    loop {
        let n = socket.read(&mut buffer).await?;
        if n == 0 {
            println!("Connection closed");
            return Ok(());
        }

        let header = PvAccessHeader::parse(&buffer[..n])?;
        println!("Received header: {:?}", header);
        // for feature in features.iter() {
        //     if feature.match_header(&header) {
        //         let mut state_guard = state.lock().unwrap(); // or .await for tokio Mutex
        //         let response = feature.handle_message(&buffer[..n], &mut *state_guard)?;
        //         let response_bytes = response.to_bytes()?;
        //         socket.write_all(&response_bytes).await?;
        //         break;
        //     }
        // }
    }
}
