use async_trait::async_trait;

use crate::pvaccess::{pv_echo::EchoResponse, pv_search::SearchResponse};

use super::{
    pv_beacon::BeaconMessage, pv_echo::EchoMessage, pv_search::SearchRequest,
    pv_validation::ConnectionValidationRequest, with_pvaccess::PVAccessServer,
};

pub enum ResponseCompletionStatusEnum {
    Ok = 0,
    Warning = 1,
    Error = 2,
    FatalError = 3,
}

pub struct ResponseCompletionStatus {
    pub response_type: ResponseCompletionStatusEnum,
    pub message: String,
    pub call_tree: Option<String>,
}

#[async_trait]
pub trait CorePvAccessHandler: Send + Sync {
    // 0x00 page 31 in spec
    async fn handle_beacon(&self, msg: BeaconMessage);
    // 0x01 page 33 in spec
    async fn handle_connection_validation(&self, msg: ConnectionValidationRequest);
    // 0x02 page 34 in spec
    async fn handle_echo(&self, msg: EchoMessage);
    // 0x03 page 35 in spec
    async fn handle_search_request(&self, msg: SearchRequest);
}

#[async_trait]
impl CorePvAccessHandler for PVAccessServer {
    async fn handle_beacon(&self, msg: BeaconMessage) {
        println!("🔹 Beacon message received: {:?}", msg);
        todo!("implement immediate response behavior");
    }

    async fn handle_connection_validation(&self, msg: ConnectionValidationRequest) {
        let _ = msg;
        println!("🔹 Connection validation request received");
        todo!("make this bigger");
    }

    async fn handle_echo(&self, msg: EchoMessage) {
        let response = EchoResponse {
            repeated_bytes: msg.random_bytes.clone(),
        };
        println!("🔹 Echo message received: {:?}", msg.random_bytes);
        todo!("implement immediate response behavior");
    }

    // 0x04 page 36 in spec
    async fn handle_search_request(&self, msg: SearchRequest) {
        let _ = msg;
        println!("Received a search request");
        let _response = SearchResponse {
            guid: todo!(),
            search_sequence_id: todo!(),
            server_address: todo!(),
            server_port: todo!(),
            protocol: "tcp".to_string(),
            found: todo!(),
            search_instance_ids: todo!(),
        };
        todo!("parse this correctly")
    }
}
