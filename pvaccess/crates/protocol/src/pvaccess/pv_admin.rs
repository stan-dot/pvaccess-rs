use async_trait::async_trait;

use super::{
    pv_core::{ResponseCompletionStatus, ResponseCompletionStatusEnum},
    with_pvaccess::PVAccessServer,
};

#[derive(Debug)]
struct DestroyRequest {
    server_channel_id: u32,
    request_id: u32,
}

impl DestroyRequest {
    fn new(server_channel_id: u32, request_id: u32) -> Self {
        Self {
            server_channel_id,
            request_id,
        }
    }

    fn from_bytes(bytes: &[u8]) -> Result<Self, std::io::Error> {
        // Deserialize bytes into DestroyRequest
        // todo This is a placeholder implementation
        let server_channel_id = u32::from_be_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]);
        let request_id = u32::from_be_bytes([bytes[4], bytes[5], bytes[6], bytes[7]]);
        Ok(Self {
            server_channel_id,
            request_id,
        })
    }
}

#[derive(Debug)]
struct ChannelProcessRequestInit {
    server_channel_id: u32,
    request_id: u32,
    subcommand: u8,
    pv_request_if: Option<String>,
    pv_request: Option<String>,
}

impl ChannelProcessRequestInit {
    fn new(server_channel_id: u32, request_id: u32) -> Self {
        Self {
            server_channel_id,
            request_id,
            subcommand: 0x08,
            pv_request: None,
            pv_request_if: None,
        }
    }
}

#[derive(Debug)]
struct ChannelProceesResponseInit {
    request_id: u32,
    status: ResponseCompletionStatus,
    subcommand: u8,
}

impl ChannelProceesResponseInit {
    fn new(request_id: u32) -> Self {
        Self {
            subcommand: 0x08,
            request_id,
            status: ResponseCompletionStatus {
                response_type: ResponseCompletionStatusEnum::Ok,
                call_tree: None,
                message: "ok".to_string(),
            },
        }
    }

    fn from_bytes(bytes: &[u8]) -> Result<Self, std::io::Error> {
        // Deserialize bytes into ChannelProceesResponseInit
        // todo This is a placeholder implementation
        let request_id = u32::from_be_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]);
        let status = ResponseCompletionStatus {
            response_type: ResponseCompletionStatusEnum::Ok,
            call_tree: None,
            message: "ok".to_string(),
        };
        Ok(Self {
            request_id,
            status,
            subcommand: 0x08,
        })
    }
}

#[derive(Debug)]
struct ChannelProcessRequest {
    server_channel_id: u32,
    request_id: u32,
    subcommand: u8, // 0x00 mask for PRCOESS, 0x10 for DESTROY
}

#[derive(Debug)]
struct ChannelProcessResponse {
    request_id: u32,
    status: ResponseCompletionStatus,
    subcommand: u8, // 0x00 mask for PRCOESS, 0x10 for DESTROY
}

#[derive(Debug)]
struct ChannelGetFieldRequest {
    server_channel_id: u32,
    request_id: u32,
    subfield_name: String,
}

impl ChannelGetFieldRequest {
    fn new(server_channel_id: u32, request_id: u32) -> Self {
        Self {
            server_channel_id,
            request_id,
            subfield_name: String::new(),
        }
    }

    fn from_bytes(bytes: &[u8]) -> Result<Self, std::io::Error> {
        // Deserialize bytes into ChannelGetFieldRequest
        // todo This is a placeholder implementation
        let server_channel_id = u32::from_be_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]);
        let request_id = u32::from_be_bytes([bytes[4], bytes[5], bytes[6], bytes[7]]);
        let subfield_name = String::from_utf8_lossy(&bytes[8..]).to_string();
        Ok(Self {
            server_channel_id,
            request_id,
            subfield_name,
        })
    }
}

#[derive(Debug)]
struct ChannelGetFieldResponse {
    request_id: u32,
    status: ResponseCompletionStatus,
    subfield_name: String,
    subcommand: u8, // 0x00 mask for PRCOESS, 0x10 for DESTROY
}

#[derive(Debug)]
enum PvMessageType {
    Info = 0,
    Warning = 1,
    Error = 2,
    FatalError = 3,
}

#[derive(Debug)]
struct PvMessage {
    request_id: u32,
    message_type: PvMessageType,
    message: String,
}

impl PvMessage {
    fn new(request_id: u32, message_type: PvMessageType, message: String) -> Self {
        Self {
            request_id,
            message_type,
            message,
        }
    }

    fn from_bytes(bytes: &[u8]) -> Result<Self, std::io::Error> {
        // Deserialize bytes into PvMessage
        // todo This is a placeholder implementation
        let request_id = u32::from_be_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]);
        let message_type = match bytes[4] {
            0 => PvMessageType::Info,
            1 => PvMessageType::Warning,
            2 => PvMessageType::Error,
            3 => PvMessageType::FatalError,
            _ => {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    "Invalid message type",
                ));
            }
        };
        let message = String::from_utf8_lossy(&bytes[5..]).to_string();
        Ok(Self {
            request_id,
            message_type,
            message,
        })
    }
}

#[derive(Debug)]
struct ChannelRPCRequestInit {
    server_channel_id: u32,
    request_id: u32,
    subcommand: u8,
    pv_request_if: Option<String>,
    pv_request: Option<String>,
}

#[derive(Debug)]
struct AdminStatus {
    open_channel_process_requests: Vec<ChannelProcessRequestInit>,
    pending_requests: Vec<ChannelProcessRequest>,
}

#[async_trait]
pub trait AdminHandler: Send + Sync {
    // 0xF page 50
    async fn handle_destroy_request(&self, bytes: &[u8]);
    // 0x10 page 51
    async fn handle_channel_process(&self, msg: &[u8]);
    // 0x11 page 52
    async fn handle_get_introspection_data(&self, msg: &[u8]);
    // 0x12 page 53
    async fn handle_message(&self, msg: &[u8]);
    // 0x14 page 53 - are those the same as 0x10?
    // async fn handle_channel_rpc(&self, msg: ChannelRPC);
    // 0x15 page 55
    // async fn handle_cancel_request(&self, msg: CancelRequest);
}

#[async_trait]
impl AdminHandler for PVAccessServer {
    async fn handle_destroy_request(&self, bytes: &[u8]) {
        let r = DestroyRequest::from_bytes(bytes).unwrap();
        // todo Handle destroy request
        println!("Handling destroy request: {:?}", r);
    }
    async fn handle_channel_process(&self, bytes: &[u8]) {
        let i = ChannelProceesResponseInit::from_bytes(bytes).unwrap();
        // todo Handle channel process request
        println!("Handling channel process request: {:?}", i);
    }
    async fn handle_get_introspection_data(&self, msg: &[u8]) {
        let r = ChannelGetFieldRequest::from_bytes(msg).unwrap();
        // todo Handle get introspection data request
        println!("Handling get introspection data request: {:?}", r);
    }
    async fn handle_message(&self, bytes: &[u8]) {
        let m = PvMessage::from_bytes(bytes).unwrap();
        // todo Handle message
        println!("Handling message: {:?}", bytes);
    }
}
