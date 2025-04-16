use futures_util::{SinkExt, StreamExt};
use protocol::pvaccess::client_manager::ClientManager;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::broadcast;
use tokio_tungstenite::{accept_async, tungstenite::Message};

/// Handles an incoming WebSocket connection
pub async fn handle_websocket_connection(
    raw_stream: TcpStream,
    mut receiver: broadcast::Receiver<String>, // Subscribe to client state updates
) {
    let ws_stream = accept_async(raw_stream)
        .await
        .expect("WebSocket accept failed");
    let (mut ws_sender, _) = ws_stream.split();

    while let Ok(update) = receiver.recv().await {
        if let Err(_) = ws_sender.send(Message::Text(update.into())).await {
            println!("🔴 Admin client disconnected");
            break;
        }
    }
}

/// Starts the WebSocket server
pub async fn start_websocket_server(manager: Arc<ClientManager>, address: SocketAddr, port: u32) {
    let url: &str = &format!("{:?}:{:?}", address, port.to_string());
    let listener = TcpListener::bind(url).await.unwrap();

    while let Ok((stream, _)) = listener.accept().await {
        let receiver = manager.broadcaster.subscribe();
        tokio::spawn(handle_websocket_connection(stream, receiver));
    }
}
