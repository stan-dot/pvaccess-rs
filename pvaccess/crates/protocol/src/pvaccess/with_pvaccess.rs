use async_trait::async_trait;
use byteorder::{BigEndian, LittleEndian, ReadBytesExt};
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::tcp::OwnedWriteHalf;
use tokio::net::{TcpListener, TcpStream};
use tokio::time::Duration;
use tokio::{net::UdpSocket, sync::Mutex, time::interval};
use uuid::Uuid;

use anyhow::{Error, Result as AResult};
use std::io::{Cursor, Result};

use crate::protocol::ProtocolServer;
use crate::pvaccess::pv_admin::AdminHandler;
use crate::pvaccess::pv_core::CorePvAccessHandler;
use crate::pvaccess::pv_validation::ConnectionValidationRequest;

use super::client_manager::ClientManager;
use super::pv_beacon::BeaconMessage;

/// 🔹 `pvAccess` Protocol Header (fixed 8-byte structure)
#[derive(Debug, Clone, Copy)]
pub struct PvAccessHeader {
    pub magic: u8,           // Always 0xCA
    pub version: u8,         // Protocol version
    pub flags: u8,           // Bitmask flags (endianness, segmentation, etc.)
    pub message_command: u8, // Message type
    pub payload_size: u32,   // Length of payload (non-aligned bytes)
}

// https://docs.epics-controls.org/en/latest/pv-access/protocol.html#version-2
// on version
impl PvAccessHeader {
    /// 🔹 Create a new header
    pub fn new(flags: u8, command: u8, payload_size: u32) -> Self {
        Self {
            magic: 0xCA,
            version: 2,
            flags,
            message_command: command,
            payload_size,
        }
    }

    /// 🔹 Deserialize from bytes
    pub fn from_bytes(bytes: &[u8]) -> Result<Self> {
        if bytes.len() < 8 {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "Header too short",
            ));
        }

        let mut cursor = Cursor::new(bytes);
        let magic = cursor.read_u8()?;
        if magic != 0xCA {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "Invalid magic byte",
            ));
        }

        let version = cursor.read_u8()?;
        let flags = cursor.read_u8()?;
        let message_command = cursor.read_u8()?;

        let payload_size = if flags & 0b1000_0000 != 0 {
            cursor.read_u32::<BigEndian>()?
        } else {
            cursor.read_u32::<LittleEndian>()?
        };

        Ok(Self {
            magic,
            version,
            flags,
            message_command,
            payload_size,
        })
    }

    /// 🔹 Serialize to bytes
    pub fn to_bytes(&self) -> Result<Vec<u8>> {
        {
            use byteorder::WriteBytesExt;
            let mut buffer = Vec::new();
            buffer.write_u8(self.magic)?;
            buffer.write_u8(self.version)?;
            buffer.write_u8(self.flags)?;
            buffer.write_u8(self.message_command)?;

            if self.flags & 0b1000_0000 != 0 {
                buffer.write_u32::<BigEndian>(self.payload_size)?;
            } else {
                buffer.write_u32::<LittleEndian>(self.payload_size)?;
            }
            Ok(buffer)
        }
    }

    /// 🔹 Check if message is segmented
    pub fn is_segmented(&self) -> bool {
        matches!(
            self.flags & 0b0011_0000,
            0b0001_0000 | 0b0010_0000 | 0b0011_0000
        )
    }

    /// 🔹 Check if message is from server
    pub fn is_server_message(&self) -> bool {
        self.flags & 0b0100_0000 != 0
    }

    /// 🔹 Check endianness
    pub fn is_big_endian(&self) -> bool {
        self.flags & 0b1000_0000 != 0
    }
}

#[test]
fn test_header_serialization() {
    let header = PvAccessHeader::new(0b0100_0000, 5, 1234); // Server message, command 5, payload 1234
    let bytes = header.to_bytes().unwrap();
    let parsed_header = PvAccessHeader::from_bytes(&bytes).unwrap();
    assert_eq!(header.magic, parsed_header.magic);
    assert_eq!(header.version, parsed_header.version);
    assert_eq!(header.flags, parsed_header.flags);
    assert_eq!(header.message_command, parsed_header.message_command);
    assert_eq!(header.payload_size, parsed_header.payload_size);
}

pub struct PVAccessServer {
    pub uuid: Uuid,
    pub messages: Arc<Mutex<Vec<PvAccessHeader>>>, // Store parsed headers
    pub connections: Arc<Mutex<Vec<SocketAddr>>>,  // Store parsed connection addresses
    channels: Arc<Mutex<Vec<String>>>,             // Store channel names
    server_port: u16,
}

#[async_trait]
impl ProtocolServer for PVAccessServer {
    type Header = PvAccessHeader;
    fn discover_message(&self) -> Vec<u8> {
        // PvAccessHeader::new(0b0000_0000, 1, 0).to_bytes().unwrap()
        BeaconMessage::new(5076, self.uuid).to_bytes().unwrap()
    }

    fn parse_header(&self, data: &[u8]) -> AResult<PvAccessHeader, Error> {
        let header = PvAccessHeader::from_bytes(data).unwrap();
        Ok(header)
        // PvAccessHeader::from_bytes(data)
        //     .map(|h| Box::new(h) as Box<dyn Any>)
        //     .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, format!("Failed to parse header: {}", e)))
    }

    async fn create_channel(&self, name: &str) -> bool {
        println!("ProtocolY: Creating channel {}", name);
        true
    }

    async fn delete_channel(&self, name: &str) -> bool {
        println!("ProtocolY: Deleting channel {}", name);
        true
    }

    async fn list_channels(&self) -> Vec<String> {
        vec!["channel1".into(), "channel2".into()]
    }

    async fn channel_put(&self, channel_name: &str, message: String) -> bool {
        println!(
            "ProtocolY: Sending message '{}' to {}",
            message, channel_name
        );
        true
    }

    async fn channel_get(&self, channel_name: &str, limit: usize) -> Vec<String> {
        println!("ProtocolY: Getting messages from {}", channel_name);
        vec!["msg1".into(), "msg2".into()]
            .into_iter()
            .take(limit)
            .collect()
    }
}

impl PVAccessServer {
    pub fn new() -> Self {
        Self {
            server_port: 5076,
            uuid: Uuid::new_v4(),
            messages: Arc::new(Mutex::new(Vec::new())),
            connections: Arc::new(Mutex::new(Vec::new())),
            channels: Arc::new(Mutex::new(Vec::new())),
        }
    }
    /// 🔹 Start sending UDP beacons every 15 seconds
    pub async fn start_udp_beacons(&self, bind_addr: &str, target_addr: &str) {
        let socket = UdpSocket::bind(bind_addr).await.unwrap();
        let mut interval = interval(Duration::from_secs(15));
        let beacon = BeaconMessage::new(self.server_port, self.uuid)
            .to_bytes()
            .unwrap();

        loop {
            interval.tick().await;
            if let Err(e) = socket.send_to(&beacon, target_addr).await {
                eprintln!("❌ Failed to send UDP beacon: {:?}", e);
            }
        }
    }
    /// 🔹 Start the TCP server and handle connection validation
    pub async fn start_tcp_server(&self, addr: String) {
        let listener = TcpListener::bind(addr).await.unwrap();

        // let standard_socket = StdSocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8080);
        let standard_socket: SocketAddr = "127.0.0.1:8080".parse().unwrap();
        let address = SocketAddr::from(standard_socket);
        while let Ok((stream, _)) = listener.accept().await {
            println!("🔹 New client connected");
            // let connections = Arc::clone(&self.connections);
            let m = ClientManager {
                clients: todo!(),
                broadcaster: todo!(),
            };
            let manager = Arc::new(m);
            let manager_clone = Arc::clone(&manager);
            // todo this task is not controlled at the moment, like in the main implementation
            tokio::spawn(self.handle_client(stream, manager_clone, address));
        }
    }

    /// 🔹 Handle a new client connection
    async fn handle_client(
        &self,
        stream: TcpStream,
        manager: Arc<ClientManager>,
        addr: SocketAddr,
    ) {
        // 🔁 Split into owned read/write halves
        let (mut reader, writer) = stream.into_split();
        let mut writer = writer; // Declare writer as mutable
        // 1️⃣ Send Connection Validation Request
        let validation_request = ConnectionValidationRequest {
            server_receive_buffer_size: 8192,
            server_introspection_registry_max_size: 128,
            auth_nz: vec!["none".into()], // No authentication for now
        };

        {
            use tokio::io::{AsyncReadExt, AsyncWriteExt};
            let validation_request_bytes = validation_request.to_bytes().unwrap();
            writer.write_all(&validation_request_bytes).await.unwrap();
            println!("✅ Sent connection validation request");

            // 2️⃣ Wait for Client's Connection Validation Response
            let mut buffer = vec![0; 1024];
            let n = reader.read(&mut buffer).await.unwrap();
            // Store the connection - only the name really
            // connections.lock().await.push(stream);
            loop {
                match reader.read(&mut buffer).await {
                    Ok(0) => {
                        println!("🔹 Client disconnected");
                        let manager = Arc::clone(&manager);
                        manager.remove_client(addr).await;

                        // Arc::clone(&manager).remove_client(addr).await;
                        return;
                    }
                    Ok(n) => {
                        let manager = Arc::clone(&manager);
                        if let Err(e) = self
                            .handle_message(&mut writer, &buffer[..n], manager, addr)
                            .await
                        {
                            eprintln!("❌ Error processing message: {}", e);
                        }
                    }
                    Err(e) => {
                        eprintln!("❌ Error reading from client: {}", e);
                        return;
                    }
                }
            }
        }
    }

    /// 🔹 Process incoming messages
    async fn handle_message(
        &self,
        writer: &mut OwnedWriteHalf,
        data: &[u8],
        manager: Arc<ClientManager>,
        addr: SocketAddr,
    ) -> AResult<(), anyhow::Error> {
        let header = PvAccessHeader::from_bytes(&data[..8])
            .map_err(|_| "Invalid header")
            .unwrap();
        let is_big_endian = header.is_big_endian();

        // todo implement
        let response = manager.verify_response(addr.to_string()).await;
        let _m = format!(
            "⚠️ Unexpected response from {:?}: {:#X}",
            addr, header.message_command
        );
        // return Error::msg(m);

        // todo this is for connection validation - need to remember per-client logic
        // let client_response = ConnectionValidationResponse::from_bytes(&buffer[..n]).unwrap();
        // println!("🔹 Client responded: {:?}", client_response);
        match header.message_command {
            0x02 => {
                let response = self.handle_echo(&data[8..], is_big_endian).await;
                println!("🔹 Echo response: {:?}", response);
                let b = response.to_bytes(is_big_endian).unwrap();
                tokio::io::AsyncWriteExt::write_all(writer, &b)
                    .await
                    .unwrap();
            }
            0x08 => self.handle_channel_process(&data[8..]).await,
            _ => println!(
                "⚠️ Unknown message received with command: {:#X}",
                header.message_command
            ),
        }
        Ok(())
    }
}
