use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::{collections::HashMap, net::SocketAddr};
use tokio::sync::{Mutex, broadcast};

/// Represents per-client state
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ClientSession {
    pub addr: SocketAddr,
    pub authenticated: bool,
    pub open_channels: Vec<String>,
}

/// Holds all active client sessions and a broadcast channel for updates
pub struct ClientManager {
    pub clients: Arc<Mutex<HashMap<String, ClientSession>>>,
    pub broadcaster: broadcast::Sender<String>, // Broadcast updates to WebSocket clients
}

impl ClientManager {
    pub fn new() -> Self {
        let (broadcaster, _) = broadcast::channel(10);
        Self {
            clients: Arc::new(Mutex::new(HashMap::new())),
            broadcaster,
        }
    }

    /// 🔹 Add a new client session
    pub async fn add_client(&self, addr: SocketAddr) {
        let mut clients = self.clients.lock().await;
        clients.insert(
            addr.to_string(),
            ClientSession {
                addr,
                authenticated: false,
                open_channels: Vec::new(),
            },
        );

        let update = serde_json::to_string(&clients.values().collect::<Vec<_>>()).unwrap();
        let _ = self.broadcaster.send(update);
    }

    /// 🔹 Get a client session by address
    pub async fn verify_response(&self, word: String) {
        println!("Verifying response: {}", word);
        todo!("Implement response verification logic")
    }

    /// 🔹 Remove a client session
    pub async fn remove_client(&self, addr: SocketAddr) {
        let mut clients = self.clients.lock().await;
        clients.remove(&addr.to_string());

        let update = serde_json::to_string(&clients.values().collect::<Vec<_>>()).unwrap();
        let _ = self.broadcaster.send(update);
    }

    /// 🔹 Update client authentication status
    pub async fn authenticate_client(&self, addr: String) {
        if let Some(client) = self.clients.lock().await.get_mut(&addr) {
            client.authenticated = true;

            let update =
                serde_json::to_string(&self.clients.lock().await.values().collect::<Vec<_>>())
                    .unwrap();
            let _ = self.broadcaster.send(update);
        }
    }
}
