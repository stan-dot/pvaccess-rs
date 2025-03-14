use futures::{SinkExt, StreamExt};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::net::{TcpListener, TcpStream, UdpSocket};
use tokio::sync::{RwLock, mpsc};
use tokio::task;
use tokio::time::{Duration, interval};
use tokio_tungstenite::tungstenite::Message;
use warp::Filter;

#[derive(Clone)]
struct ServerState {
    clients: Arc<RwLock<HashMap<String, String>>>, // ClientID -> Subscribed Channel
    ws_clients: Arc<RwLock<Vec<warp::ws::WebSocket>>>,
}

impl ServerState {
    async fn get_clients(&self) -> Vec<String> {
        let clients = self.clients.read().await;
        clients.keys().cloned().collect()
    }

    async fn broadcast_ws(&self, message: &str) {
        let mut clients = self.ws_clients.write().await;
        clients.retain(|ws| {
            if let Err(_) = ws.send(Message::text(message.to_string())) {
                false
            } else {
                true
            }
        });
    }
}

// TCP Client Handler
async fn handle_tcp_client(mut socket: TcpStream, state: ServerState) {
    let mut buffer = vec![0; 1024];
    let client_id = format!("{:?}", socket.peer_addr().unwrap());

    while let Ok(n) = socket.read(&mut buffer).await {
        if n == 0 {
            println!("Client {} disconnected", client_id);
            break;
        }

        let message = String::from_utf8_lossy(&buffer[..n]).to_string();
        println!("TCP Client {} sent: {}", client_id, message);

        if message.starts_with("SUBSCRIBE") {
            let parts: Vec<&str> = message.split_whitespace().collect();
            if parts.len() == 2 {
                let mut clients = state.clients.write().await;
                clients.insert(client_id.clone(), parts[1].to_string());

                state
                    .broadcast_ws(&format!("{} subscribed to {}", client_id, parts[1]))
                    .await;
            }
        }
    }
}

// WebSocket Handler
async fn websocket_handler(ws: warp::ws::Ws, state: ServerState) -> impl warp::Reply {
    ws.on_upgrade(move |socket| handle_ws(socket, state))
}

async fn handle_ws(ws: warp::ws::WebSocket, state: ServerState) {
    state.ws_clients.write().await.push(ws);
}

// UDP Beacon
async fn udp_beacon(state: ServerState) {
    let socket = UdpSocket::bind("0.0.0.0:0").await.unwrap();
    let udp_addr = "127.0.0.1:12345";
    let mut ticker = interval(Duration::from_secs(15));

    for _ in 0..15 {
        let message = b"UDP Beacon: Server is live!";
        let _ = socket.send_to(message, udp_addr).await;
        ticker.tick().await;
    }

    let mut long_term_ticker = interval(Duration::from_secs(60));
    loop {
        let message = b"UDP Beacon: Still alive!";
        let _ = socket.send_to(message, udp_addr).await;
        long_term_ticker.tick().await;
    }
}

// Worker Pool
async fn worker_pool(mut rx: mpsc::Receiver<String>) {
    while let Some(task) = rx.recv().await {
        println!("Processing message: {}", task);
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
}

#[tokio::main]
async fn main() {
    let state = ServerState {
        clients: Arc::new(RwLock::new(HashMap::new())),
        ws_clients: Arc::new(RwLock::new(Vec::new())),
    };

    // TCP Server
    let tcp_state = state.clone();
    let tcp_task = task::spawn(async move {
        let listener = TcpListener::bind("127.0.0.1:8000").await.unwrap();
        println!("TCP Server running on 127.0.0.1:8000");

        while let Ok((socket, _)) = listener.accept().await {
            let state = tcp_state.clone();
            task::spawn(handle_tcp_client(socket, state));
        }
    });

    // WebSocket Server
    let ws_state = state.clone();
    let ws_route = warp::path("ws")
        .and(warp::ws())
        .and(warp::any().map(move || ws_state.clone()))
        .map(|ws, state| websocket_handler(ws, state));

    let ws_task = task::spawn(async move {
        warp::serve(ws_route).run(([127, 0, 0, 1], 3030)).await;
    });

    // UDP Beacon Task
    let udp_state = state.clone();
    let udp_task = task::spawn(udp_beacon(udp_state));

    // Worker Pool
    let (tx, rx) = mpsc::channel(100);
    for _ in 0..4 {
        let worker_rx = rx.clone();
        task::spawn(worker_pool(worker_rx));
    }

    // Test sending messages to worker pool
    for i in 0..10 {
        let _ = tx.send(format!("Message {}", i)).await;
    }

    tokio::try_join!(tcp_task, ws_task, udp_task).unwrap();
}
