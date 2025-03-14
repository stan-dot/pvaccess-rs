use std::collections::{HashMap, VecDeque};
use std::sync::Arc;
use tokio::io::AsyncReadExt;
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::RwLock;
use tokio::sync::mpsc::{Receiver, Sender, channel};

#[derive(Clone)]
struct Server {
    channels: Arc<RwLock<HashMap<String, Arc<RwLock<Channel>>>>>,
    clients: Arc<RwLock<HashMap<String, Sender<String>>>>,
}

#[derive(Clone)]
struct Channel {
    name: String,
    buffer: Arc<RwLock<VecDeque<String>>>,
    subscribers: Arc<RwLock<HashMap<String, Sender<String>>>>,
}

impl Channel {
    fn new(name: &str, size: usize) -> Self {
        Channel {
            name: name.to_string(),
            buffer: Arc::new(RwLock::new(VecDeque::with_capacity(size))),
            subscribers: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    async fn publish(&self, message: String) {
        let mut buffer = self.buffer.write().await;
        if buffer.len() == buffer.capacity() {
            buffer.pop_front(); // Remove the oldest message if buffer is full
        }
        buffer.push_back(message.clone()); // Add the new message to the buffer

        // Notify all subscribers with the new message
        let subscribers = self.subscribers.read().await;
        for (client_id, sender) in subscribers.iter() {
            if let Err(_) = sender.send(message.clone()).await {
                // Handle client disconnection (or failure)
                println!("Failed to send message to client: {}", client_id);
            }
        }
    }

    async fn subscribe(&self, client_id: String, sender: Sender<String>) {
        let mut subscribers = self.subscribers.write().await;
        subscribers.insert(client_id, sender);
    }

    async fn get_latest_messages(&self, num: usize) -> Vec<String> {
        let buffer = self.buffer.read().await;
        buffer.iter().rev().take(num).cloned().collect()
    }
}

impl Server {
    fn new() -> Self {
        Server {
            channels: Arc::new(RwLock::new(HashMap::new())),
            clients: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    async fn get_or_create_channel(&self, name: &str, buffer_size: usize) -> Arc<RwLock<Channel>> {
        let mut channels = self.channels.write().await;
        if !channels.contains_key(name) {
            channels.insert(
                name.to_string(),
                Arc::new(RwLock::new(Channel::new(name, buffer_size))),
            );
        }
        channels.get(name).unwrap().clone()
    }

    async fn handle_client(&self, client_id: String, mut socket: TcpStream) {
        let (tx, mut rx): (Sender<String>, Receiver<String>) = channel(32);

        // Register client for future messages
        {
            let mut clients = self.clients.write().await;
            clients.insert(client_id.clone(), tx);
        }

        // Handle receiving subscription commands
        let mut buffer = vec![0; 1024];
        loop {
            match socket.read(&mut buffer).await {
                Ok(0) => {
                    println!("Client {} disconnected", client_id);
                    break;
                }
                Ok(n) => {
                    let command = String::from_utf8_lossy(&buffer[..n]).to_string();
                    println!("Received command from {}: {}", client_id, command);

                    // Parse subscription commands (e.g., "SUBSCRIBE channel_name")
                    if command.starts_with("SUBSCRIBE") {
                        let parts: Vec<&str> = command.split_whitespace().collect();
                        if parts.len() == 2 {
                            let channel_name = parts[1];
                            let channel = self.get_or_create_channel(channel_name, 10).await;
                            channel.subscribe(client_id.clone(), tx.clone()).await;
                            println!(
                                "Client {} subscribed to channel: {}",
                                client_id, channel_name
                            );
                        }
                    }
                }
                Err(_) => {
                    println!("Error reading from client: {}", client_id);
                    break;
                }
            }
        }
    }

    async fn publish_to_channel(&self, channel_name: &str, message: String) {
        if let Some(channel) = self.channels.read().await.get(channel_name) {
            let c = Arc::get_mut(&mut channel).unwrap().blocking_read();
            channel.publish(message).await;
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let server = Server::new();

    // Start TCP server
    let listener = TcpListener::bind("127.0.0.1:8000").await?;
    println!("Server listening on 127.0.0.1:8000");

    loop {
        let (socket, addr) = listener.accept().await?;
        println!("New client connected: {}", addr);

        let client_id = format!("{}", addr);
        tokio::spawn(server.handle_client(client_id, socket));
    }
}
