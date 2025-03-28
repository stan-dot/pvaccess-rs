use tokio::net::{TcpListener, TcpStream};
use tokio_tungstenite::accept_async;
use futures_util::{StreamExt, SinkExt};
use std::sync::Arc;
use tokio::sync::broadcast;

/// Handles an incoming WebSocket connection
async fn handle_websocket_connection(
    raw_stream: TcpStream,
    mut receiver: broadcast::Receiver<String>, // Subscribe to client state updates
) {
    let ws_stream = accept_async(raw_stream).await.expect("WebSocket accept failed");
    let (mut ws_sender, _) = ws_stream.split();

    while let Ok(update) = receiver.recv().await {
        if let Err(_) = ws_sender.send(tokio_tungstenite::tungstenite::Message::Text(update)).await {
            println!("🔴 Admin client disconnected");
            break;
        }
    }
}

/// Starts the WebSocket server
async fn start_websocket_server(manager: Arc<ClientManager>, address: str, port: int) {
    let url: str = format!("{:?}:{:?}", address, port.toString());
    let listener = TcpListener::bind(url).await.unwrap();

    while let Ok((stream, _)) = listener.accept().await {
        let receiver = manager.broadcaster.subscribe();
        tokio::spawn(handle_websocket_connection(stream, receiver));
    }
}
