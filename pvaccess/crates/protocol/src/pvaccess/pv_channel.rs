use async_trait::async_trait;
use bitvec::vec::BitVec;

use super::pv_core::ResponseCompletionStatus;

pub struct ChannelGetRequestInit {
    server_channel_id: u32,
    request_id: u32,
    subcommand: u8, // 0x08 for init
    pv_request_if: String, //todo find the type for fielddesc
    pv_request: String // this should be pvfield
}

pub struct ChannelGetResponseInit {
    request_id: u32,
    subcommand: u8, // 0x04
    status: ResponseCompletionStatus,
    pv_structure_if: String // this should be FieldDesc
}


pub struct ChannelGetRequest{
    server_channel_id: u32,
    request_id: u32,
    subcommand: u8, // 0x40 for for GET, 0x10 for destroy
}

// changed_bit_set: BitVec,

pub struct ChannelGetResponse{
    request_id: u32,
    subcommand: u8, // 0x04
    status: ResponseCompletionStatus,
    changed_bit_set: Option<BitVec>,
    pv_structure_data: Vec<u8>, // this should be pvfield
}


pub struct ChannelPutRequestInit{
    server_channel_id: u32,
    request_id: u32,
    subcommand: u8, // 0x08 for init
    pv_request_if: String, //todo find the type for fielddesc
    pv_request: String // this should be pvfield
}

pub struct ChannelPutResponseInit{
    request_id: u32,
    subcommand: u8, // 0x04
    status: ResponseCompletionStatus,
    pv_structure_if: Option<String> // this should be FieldDesc
}


pub struct ChannelPutRequest {
    server_channel_id: u32,
    request_id: u32,
    subcommand: u8, // 0x00 for for PUT, 0x10 for destroy
    pv_structure_data: Vec<u8>, // this should be pvfield
    to_put_bit_set: BitVec,
}

pub struct ChannelPutResponse{
    request_id: u32,
    subcommand: u8, // 0x04
    status: ResponseCompletionStatus,
}

pub struct DestroyChannelRequest {
    client_channel_id: u32,
    server_channel_id: u32,
}

pub struct DestroyChannelResponse {
    client_channel_id: u32,
    server_channel_id: u32,
}

pub struct ChannelInit {
    client_channel_id: u32,
    channel_name: String,
}

pub struct CreateChannelRequest {
    channels: Vec<ChannelInit>,
}

pub struct CreateChannelResponse {
    client_channel_id: u32,
    server_channel_id: u32,
    status: ResponseCompletionStatus,
}


pub struct ChannelMonitorRequestInit{
    server_channel_id: u32,
    request_id: u32,
    subcommand: u8, // 0x80 pipeline support for init
    pv_request_if: String, //todo find the type for fielddesc
    pv_request: String, // this should be pvfield
    queue_size: Option<u32>,
}


pub struct ChannelMonitorResponseInit{
    request_id: u32,
    subcommand: u8, // 0x04
    status: ResponseCompletionStatus,
    pv_structure_if: Option<String>, // this should be FieldDesc
}


#[async_trait]
pub trait ChannelHandler {
    // 0x07 page 37
    async fn handle_create(&self, msg: CreateChannelRequest);
    // 0x08 page 38
    async fn handle_destroy(&self, msg: DestroyChannelRequest);
    // 0x0A page 39
    async fn handle_get(&self, msg:ChannelGetRequestInit);
    // 0x0B page 41
    async fn handle_put(&self, msg:ChannelPutRequestInit);
    // 0x0C page 42
    // async fn handle_put_get(&self, msg: ChannelPutGet);
    // 0x0D page 45
    // async fn handle_monitor(&self, msg: ChannelMonitor);
    // 0x0E page 47
    // async fn handle_array(&self, msg: ChannelArray);
}
