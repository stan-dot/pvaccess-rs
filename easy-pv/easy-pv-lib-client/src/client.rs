use crate::config::ClientConfig;

use tokio::{signal, sync::oneshot};

pub async fn start_client(config: ClientConfig) {
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

    // Perform Graceful Shutdown
    // udp_task.abort(); // Stop accepting new UDP clients
    // tcp_task.abort(); // Stop accepting new TCP clients
    println!("Server shut down gracefully.");
}
