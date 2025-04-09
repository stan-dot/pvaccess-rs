use async_trait::async_trait;


#[async_trait]
pub trait AdminHandler : Send + Sync {
    // 0xF page 50
    // async fn handle_destroy_request(&self, msg: DestroyRequest);
    // // 0x10 page 51
    // async fn handle_channel_process(&self, msg: ChannelProcess);
    // // 0x11 page 52
    // async fn handle_get_introspection_data(&self, msg: GetIntrospectionData);
    // // 0x12 page 53
    // async fn handle_message(&self, msg: PvMessage);
    // // 0x14 page 53
    // async fn handle_channel_rpc(&self, msg: ChannelRPC);
    // // 0x15 page 55
    // async fn handle_cancel_request(&self, msg: CancelRequest);
}