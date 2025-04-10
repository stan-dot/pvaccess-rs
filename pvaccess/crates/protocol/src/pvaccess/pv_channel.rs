use async_trait::async_trait;

#[async_trait]
pub trait ChannelHandler {
    // 0x07 page 37
    // async fn handle_create(&self, msg: CreateChannel);
    // // 0x08 page 38
    // async fn handle_destroy(&self, msg: DestroyChannelRequest);
    // // 0x0A page 39
    // async fn handle_get(&self, msg: ChannelGet);
    // // 0x0B page 40
    // async fn handle_put(&self, msg: ChannelPut);
    // // 0x0C page 42
    // async fn handle_put_get(&self, msg: ChannelPutGet);
    // // 0x0D page 45
    // async fn handle_monitor(&self, msg: ChannelMonitor);
    // // 0x0E page 47
    // async fn handle_array(&self, msg: ChannelArray);
}

pub struct DestroyChannelRequest {
    client_channel_id: u32,
    server_channel_id: u32,
}

pub struct DestroyChannelResponse {
    client_channel_id: u32,
    server_channel_id: u32,
}

pub struct CreateChannelRequest {
    // channels: Vec<{client_channel_id: u32, channel_name:string}>
}
