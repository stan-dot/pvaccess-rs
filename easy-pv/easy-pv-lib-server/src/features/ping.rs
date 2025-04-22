use async_trait::async_trait;
use bincode::config::Endianness;
use easy_pv_datatypes::header::PvAccessHeader;
use tokio::sync::MutexGuard;

use crate::{async_feature::Feature, state::ServerState};

struct PingRequest {
    // Define the fields for the PingRequest
}

impl TryInto<PingRequest> for &[u8] {
    type Error = anyhow::Error;

    fn try_into(self) -> Result<PingRequest, Self::Error> {
        // Implement the logic to parse raw bytes into PingRequest
        PingRequest::from_bytes(self)
    }
}

impl TryFrom<&[u8]> for PingRequest {
    type Error = anyhow::Error;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        PingRequest::from_bytes(value)
    }
}


impl PingRequest {
    fn from_bytes(raw: &[u8]) -> Result<Self, anyhow::Error> {
        // Implement the logic to parse raw bytes into PingRequest
        Ok(PingRequest {})
    }
}
struct PingResponse {
    message: String,
}
impl PingResponse {
    fn to_bytes(&self) -> Vec<u8> {
        // Implement the logic to serialize PingResponse to bytes
        self.message.as_bytes().to_vec()
    }
}

pub struct PingFeature;

#[async_trait]
impl Feature for PingFeature {
    fn name(&self) -> &'static str {
        "ping"
    }

    fn match_header(&self, header: &PvAccessHeader) -> bool {
        header.msg_type == 0x01
    }

    // todo make the types
    type Incoming = PingRequest;
    type Outgoing = PingResponse;

    fn parse(&self, raw: &[u8], _endianness: Endianness) -> Option<Self::Incoming> {
        PingRequest::from_bytes(raw).ok()
    }

    fn serialize(&self, msg: Self::Outgoing) -> Vec<u8> {
        msg.to_bytes()
    }

    async fn handle_message<'a>(
        &self,
        msg: Self::Incoming,
        state: &mut MutexGuard<'a, ServerState>,
    ) -> Result<Self::Outgoing, anyhow::Error> {
        // Handle the message and update the server state
        // For example, increment a ping count in the server state
        state.ping_count += 1;
        Ok(PingResponse {
            message: "pong".into(),
        })
    }

    // async fn handle_message(
    //     &self,
    //     _msg: PingRequest,
    //     state: &mut MutexGuard<'_, ServerState>,
    // ) -> Result<PingResponse, anyhow::Error> {
    //     state.ping_count += 1;
    //     Ok(PingResponse {
    //         message: "pong".into(),
    //     })
    // }
}
