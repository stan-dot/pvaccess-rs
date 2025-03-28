use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::net::TcpStream;

/// Represents per-client state
struct ClientSession {
    pub stream: TcpStream,
    // todo adjust that for the pv types
    pub last_sent_request: Option<u8>, // Last message command sent (e.g., 0x01 for validation)
    pub expected_response: Option<u8>, // Expected response message command
    pub authenticated: bool, // Track authentication state
    pub open_channels: Vec<String>, // List of active channels
}

/// Holds all active client sessions
struct ClientManager {
    pub clients: Arc<Mutex<HashMap<SocketAddr, ClientSession>>>,
}

impl ClientManager {
    pub fn new() -> Self {
        Self {
            clients: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub async fn add_client(&self, addr: SocketAddr, stream: TcpStream) {
        let session = ClientSession {
            stream,
            last_sent_request: None,
            expected_response: None,
            authenticated: false,
            open_channels: Vec::new(),
        };
        self.clients.lock().await.insert(addr, session);
    }

    pub async fn remove_client(&self, addr: &SocketAddr) {
        self.clients.lock().await.remove(addr);
    }

    pub async fn set_expected_response(&self, addr: &SocketAddr, expected: u8) {
        if let Some(client) = self.clients.lock().await.get_mut(addr) {
            client.expected_response = Some(expected);
        }
    }

    pub async fn verify_response(&self, addr: &SocketAddr, response: u8) -> bool {
        if let Some(client) = self.clients.lock().await.get_mut(addr) {
            if let Some(expected) = client.expected_response {
                if expected == response {
                    client.expected_response = None; // Reset expectation after valid response
                    return true;
                }
            }
        }
        false
    }
}
