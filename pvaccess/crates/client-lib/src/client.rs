use config::{Config, File};
use protocol::{Msg, MsgType};
use rmp_serde::{decode, encode};
use std::collections::HashMap;
use std::env;
use std::net::{IpAddr, SocketAddr};
use std::str;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio::net::UdpSocket;

#[derive(Debug)]
pub struct Client {
    udp_port: u16,
    tcp_port: u16,
    buffer_size: usize,          // 🔹 Configurable buffer size
    server_addr: Option<String>, // Stores the discovered server address
    udp_bind_addr: String,       // 🔹 Allows multiple clients from the same machine
}

impl Client {
    /// 🔹 Create a new client instance from config
    pub fn new(
        udp_port: Option<u16>,
        tcp_port: Option<u16>,
        buffer_size: Option<usize>,
        server_addr: Option<String>,
        udp_bind_addr: Option<String>, // New: allows binding to a unique local port
    ) -> Self {
        // 🔹 Load Config
        let config_path =
            env::var("CONFIG_PATH").unwrap_or_else(|_| "crates/client/config/client".to_string());
        println!("📄 Loading config from: {}", config_path);

        let settings = Config::builder()
            .add_source(File::with_name(&config_path))
            .build()
            .expect("❌ Failed to load configuration");

        let network: HashMap<String, String> = settings.get("network").unwrap();
        // ✅ Use the config value unless overridden by the function argument
        let udp_port = udp_port.unwrap_or_else(|| network["udp_port"].parse().unwrap());
        let tcp_port = tcp_port.unwrap_or_else(|| network["tcp_port"].parse().unwrap());
        let buffer_size =
            buffer_size.unwrap_or_else(|| network["buffer_size"].parse().unwrap_or(1024));
        let server_addr = server_addr.or_else(|| network.get("server_addr").cloned());
        let udp_bind_addr = udp_bind_addr.unwrap_or_else(|| {
            network
                .get("udp_bind_addr")
                .cloned()
                .unwrap_or("0.0.0.0:0".to_string())
        });

        println!(
            "✅ Config loaded. UDP Port: {}, TCP Port: {}, Buffer Size: {}",
            udp_port, tcp_port, buffer_size
        );

        Self {
            udp_port,
            tcp_port,
            buffer_size,
            server_addr,
            udp_bind_addr,
        }
    }

    /// 🔹 Discover the TCP server via UDP broadcast
    pub async fn discover_server(&mut self) -> Option<String> {
        println!("🔍 Discovering server at UDP port {}...", self.udp_port);

        // ✅ Convert bind address to a valid `SocketAddr`
        let bind_ip: IpAddr = self
            .udp_bind_addr
            .parse()
            .expect("Invalid UDP bind address");
        let bind_addr = SocketAddr::new(bind_ip, self.udp_port);

        // ✅ Bind the UDP socket correctly
        let socket = UdpSocket::bind(bind_addr).await.unwrap();
        socket.set_broadcast(true).unwrap();  // Allows receiving broadcast packets
        let mut buffer = vec![0; self.buffer_size]; // 🔹 Use buffer size from config

        loop {
            if let Ok((size, _)) = socket.recv_from(&mut buffer).await {
                let msg = str::from_utf8(&buffer[..size]).unwrap();
                if msg.starts_with("DISCOVER_SERVER:") {
                    let server_addr = msg.replace("DISCOVER_SERVER:", "").trim().to_string();
                    self.server_addr = Some(server_addr.clone());
                    println!("✅ Discovered TCP Server: {}", server_addr);
                    return Some(server_addr);
                }
            }
        }
    }

    /// 🔹 Connect to the discovered TCP server and listen for messages.
    pub async fn connect_and_listen(&self) -> Result<(), String> {
        let server_addr = match &self.server_addr {
            Some(addr) => addr.clone(),
            None => return Err("No server address found. Run `discover_server()` first.".into()),
        };

        let mut stream = TcpStream::connect(&server_addr)
            .await
            .map_err(|e| e.to_string())?;
        println!("✅ Connected to TCP server: {}", server_addr);

        // Receive initial validation message
        let mut buffer = vec![0; self.buffer_size]; // 🔹 Use buffer size from config
        let n = stream.read(&mut buffer).await.map_err(|e| e.to_string())?;
        if let Ok(msg) = decode::from_read::<_, Msg>(&buffer[..n]) {
            println!("🔹 Received validation message: {:?}", msg);
        }

        // Send an Echo Message
        let echo_msg = Msg {
            msg_type: MsgType::Echo,
            content: "Hello, Server!".to_string(),
        };

        let mut buf = Vec::new();
        encode::write(&mut buf, &echo_msg).unwrap();
        stream.write_all(&buf).await.map_err(|e| e.to_string())?;

        // Keep listening for new messages
        self.listen_for_messages(stream).await
    }

    /// 🔹 Listen for incoming messages indefinitely.
    async fn listen_for_messages(&self, mut stream: TcpStream) -> Result<(), String> {
        let mut buffer = vec![0; self.buffer_size];

        loop {
            tokio::select! {
                res = stream.read(&mut buffer) => {
                    match res {
                        Ok(0) => {
                            println!("🚨 Server closed the connection.");
                            return Err("Server disconnected".into());
                        }
                        Ok(n) => {
                            if let Ok(msg) = decode::from_read::<_, Msg>(&buffer[..n]) {
                                println!("📩 Received message: {:?}", msg);
                            } else {
                                eprintln!("❌ Failed to decode message");
                            }
                        }
                        Err(e) => {
                            return Err(format!("❌ Read error: {}", e));
                        }
                    }
                }
                _ = Client::wait_for_shutdown() => {
                    println!("🔻 Received SIGTERM, exiting...");
                    return Ok(());
                }
            }
        }
    }

    /// 🔹 Wait for SIGTERM before exiting.
    async fn wait_for_shutdown() {
        tokio::signal::ctrl_c()
            .await
            .expect("Failed to listen for SIGINT");
        println!("🔻 Received SIGINT, closing connection.");
    }
}
