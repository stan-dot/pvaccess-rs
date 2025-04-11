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
    async fn handle_destroy_request(&self, msg: DestroyRequest);
    // 0x10 page 51
    async fn handle_channel_process(&self, msg: ChannelProcessRequestInit);
    // 0x11 page 52
    async fn handle_get_introspection_data(&self, msg: ChannelGetFieldRequest);
    // 0x12 page 53
    async fn handle_message(&self, msg: PvMessage);
    // 0x14 page 53 - are those the same as 0x10?
    // async fn handle_channel_rpc(&self, msg: ChannelRPC);
    // 0x15 page 55
    // async fn handle_cancel_request(&self, msg: CancelRequest);
}

#[async_trait]
impl AdminHandler for PVAccessServer {
    async fn handle_destroy_request(&self, msg: DestroyRequest) {
        // Handle destroy request
        println!("Handling destroy request: {:?}", msg);
    }
    async fn handle_channel_process(&self, msg: ChannelProcessRequestInit) {
        // Handle channel process request
        println!("Handling channel process request: {:?}", msg);
    }
    async fn handle_get_introspection_data(&self, msg: ChannelGetFieldRequest) {
        // Handle get introspection data request
        println!("Handling get introspection data request: {:?}", msg);
    }
    async fn handle_message(&self, msg: PvMessage) {
        // Handle message
        println!("Handling message: {:?}", msg);
    }
}
