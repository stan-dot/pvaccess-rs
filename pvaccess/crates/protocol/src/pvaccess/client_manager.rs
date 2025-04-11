use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::{
    net::unix::SocketAddr,
    sync::{Mutex, broadcast},
};

/// Represents per-client state
#[derive(Debug, Serialize, Deserialize, Clone)]
struct ClientSession {
    pub addr: String,
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
    pub async fn add_client(&self, addr: String) {
        let mut clients = self.clients.lock().await;
        clients.insert(
            addr.clone(),
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
        todo!("Implement response verification logic")
    }

    /// 🔹 Remove a client session
    pub async fn remove_client(&self, addr: String) {
        let mut clients = self.clients.lock().await;
        clients.remove(&addr);

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
