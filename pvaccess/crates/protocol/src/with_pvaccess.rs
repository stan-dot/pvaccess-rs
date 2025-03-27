use crate::pv_echo::{EchoMessage, EchoResponse};
use crate::pv_validation::{ConnectionValidationRequest, ConnectionValidationResponse};
use crate::{protocol::Protocol, pv_beacon::BeaconMessage};

use async_trait::async_trait;
use byteorder::{BigEndian, LittleEndian, ReadBytesExt, WriteBytesExt};
use std::any::Any;
use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::time::Duration;
use tokio::{
    net::{UdpSocket, unix::SocketAddr},
    sync::Mutex,
    time::interval,
};

use std::io::{Cursor, Result};

/// 🔹 `pvAccess` Protocol Header (fixed 8-byte structure)
#[derive(Debug, Clone, Copy)]
pub struct PvAccessHeader {
    pub magic: u8,           // Always 0xCA
    pub version: u8,         // Protocol version
    pub flags: u8,           // Bitmask flags (endianness, segmentation, etc.)
    pub message_command: u8, // Message type
    pub payload_size: u32,   // Length of payload (non-aligned bytes)
}

impl PvAccessHeader {
    /// 🔹 Create a new header
    pub fn new(flags: u8, command: u8, payload_size: u32) -> Self {
        Self {
            magic: 0xCA,
            version: 1, // TODO: Read from spec for correct version
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
pub struct PVAccess {
    pub messages: Arc<Mutex<Vec<PvAccessHeader>>>, // Store parsed headers
}

#[async_trait]
impl Protocol for PVAccess {
    fn discover_message(&self) -> Vec<u8> {
        // PvAccessHeader::new(0b0000_0000, 1, 0).to_bytes().unwrap()
        BeaconMessage::new(5076).to_bytes().unwrap()
    }

    fn parse_header(&self, data: &[u8]) -> Result<Box<dyn Any>, String> {
        PvAccessHeader::from_bytes(data)
            .map(|h| Box::new(h) as Box<dyn Any>)
            .map_err(|_| "Failed to parse header".to_string())
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

impl PVAccess {
    /// 🔹 Start sending UDP beacons every 15 seconds
    pub async fn start_udp_beacons(&self, bind_addr: &str, target_addr: &str) {
        let socket = UdpSocket::bind(bind_addr).await.unwrap();
        let target: SocketAddr = target_addr.parse().unwrap();
        let mut interval = interval(Duration::from_secs(15));
        let beacon = BeaconMessage::new(5076).to_bytes().unwrap();

        loop {
            interval.tick().await;
            if let Err(e) = socket.send_to(&beacon, target).await {
                eprintln!("❌ Failed to send UDP beacon: {:?}", e);
            }
        }
    }
    /// 🔹 Start the TCP server and handle connection validation
    pub async fn start_tcp_server(&self, addr: &str) {
        let listener = TcpListener::bind(addr).await.unwrap();
        println!("🔗 Server listening on {}", addr);

        while let Ok((stream, _)) = listener.accept().await {
            println!("🔹 New client connected");
            let connections = Arc::clone(&self.connections);
            tokio::spawn(Self::handle_client(stream, connections));
        }
    }

    /// 🔹 Handle a new client connection
    async fn handle_client(mut stream: TcpStream, connections: Arc<Mutex<Vec<TcpStream>>>) {
        // 1️⃣ Send Connection Validation Request
        let validation_request = ConnectionValidationRequest {
            server_receive_buffer_size: 8192,
            server_introspection_registry_max_size: 128,
            auth_nz: vec!["none".into()], // No authentication for now
        };

        let request_bytes = validation_request.to_bytes().unwrap();
        stream.write_all(&request_bytes).await.unwrap();
        println!("✅ Sent connection validation request");

        // 2️⃣ Wait for Client's Connection Validation Response
        let mut buffer = vec![0; 1024];
        let n = stream.read(&mut buffer).await.unwrap();
        // Store the connection
        connections.lock().await.push(stream);
        loop {
            match stream.read(&mut buffer).await {
                Ok(0) => {
                    println!("🔹 Client disconnected");
                    return;
                }
                Ok(n) => {
                    if let Err(e) = Self::handle_message(&mut stream, &buffer[..n]).await {
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

    /// 🔹 Process incoming messages
    async fn handle_message(stream: &mut TcpStream, data: &[u8]) -> Result<(), String> {
        let header = PvAccessHeader::from_bytes(&data[..8]).map_err(|_| "Invalid header")?;
        let is_big_endian = header.is_big_endian();

        // todo this is for connection validation - need to remember per-client logic
        // let client_response = ConnectionValidationResponse::from_bytes(&buffer[..n]).unwrap();
        // println!("🔹 Client responded: {:?}", client_response);
        todo!("make this for all the message types - incrementally according to the todo")
        match header.message_command {
            0x02 => Self::handle_echo(stream, &data[8..], is_big_endian).await?,
            _ => println!("⚠️ Unknown message received"),
        }
        Ok(())
    }
}

#[test]
fn test_echo_message_serialization() {
    let message = EchoMessage {
        random_bytes: vec![1, 2, 3, 4, 5],
    };

    let bytes_le = message.to_bytes(false).unwrap();
    let parsed_le = EchoMessage::from_bytes(&bytes_le, false).unwrap();
    assert_eq!(message.random_bytes, parsed_le.random_bytes);

    let bytes_be = message.to_bytes(true).unwrap();
    let parsed_be = EchoMessage::from_bytes(&bytes_be, true).unwrap();
    assert_eq!(message.random_bytes, parsed_be.random_bytes);
}

#[test]
fn test_echo_response_serialization() {
    let response = EchoResponse {
        repeated_bytes: vec![9, 8, 7, 6, 5],
    };

    let bytes_le = response.to_bytes(false).unwrap();
    let parsed_le = EchoResponse::from_bytes(&bytes_le, false).unwrap();
    assert_eq!(response.repeated_bytes, parsed_le.repeated_bytes);

    let bytes_be = response.to_bytes(true).unwrap();
    let parsed_be = EchoResponse::from_bytes(&bytes_be, true).unwrap();
    assert_eq!(response.repeated_bytes, parsed_be.repeated_bytes);
}

#[async_trait]
pub trait CorePvAccessHandler {
    // 0x00 page 31 in spec
    async fn handle_beacon(&self, msg: BeaconMessage);
    // 0x01 page 33 in spec
    async fn handle_connection_validation(&self, msg: ConnectionValidationRequest);
    // 0x02 page 34 in spec
    async fn handle_echo(&self, msg: EchoMessage);
    // 0x03 page 35 in spec
    async fn handle_search_request(&self, msg: SearchRequest);
    // 0x04 page 36 in spec
    async fn handle_search_response(&self, msg: SearchResponse);
}

impl CorePvAccessHandler for PVAccess{

    async fn handle_beacon(&self, msg: BeaconMessage){};
    async fn handle_connection_validation(&self, msg: ConnectionValidationRequest){};

    async fn handle_echo(&self, msg: EchoMessage){
        let _ = msg;
        todo!("implement");

    };
    async fn handle_search_request(&self, msg: SearchRequest){};
    async fn handle_search_response(&self, msg: SearchResponse){};
}


#[async_trait]
pub trait ChannelHandler {
    // 0x07 page 37
    async fn handle_create(&self, msg: CreateChannel);

    // 0x08 page 38
    async fn handle_destroy(&self, msg: DestroyChannel);
    // 0x0A page 39
    async fn handle_get(&self, msg: ChannelGet);
    // 0x0B page 40
    async fn handle_put(&self, msg: ChannelPut);
    // 0x0C page 42
    async fn handle_put_get(&self, msg: ChannelPutGet);
    // 0x0D page 45
    async fn handle_monitor(&self, msg: ChannelMonitor);
    // 0x0E page 47
    async fn handle_array(&self, msg: ChannelArray);
}

#[async_trait]
pub trait AdminHandler {
    // 0xF page 50
    async fn handle_destroy_request(&self, msg: DestroyRequest);
    // 0x10 page 51
    async fn handle_channel_process(&self, msg: ChannelProcess);
    // 0x11 page 52
    async fn handle_get_introspection_data(&self, msg: GetIntrospectionData);
    // 0x12 page 53
    async fn handle_message(&self, msg: PvMessage);
    // 0x14 page 53
    async fn handle_channel_rpc(&self, msg: ChannelRPC);
    // 0x15 page 55
    async fn handle_cancel_request(&self, msg: CancelRequest);
}
