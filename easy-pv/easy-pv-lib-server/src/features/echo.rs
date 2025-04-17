use bincode::config::Endianness;
use easy_pv_datatypes::header::PvAccessHeader;

use crate::{async_feature::Feature, state::ServerState};

pub struct ResponseMessage {
    pub message: Vec<u8>,
}
pub struct EchoFeature {}

#[async_trait::async_trait]
impl Feature for EchoFeature {
    type Incoming = Vec<u8>;
    type Outgoing = Vec<u8>;

    fn name(&self) -> &'static str {
        "Echo"
    }

    fn parse(&self, raw: &[u8], _endianness: Endianness) -> Option<Self::Incoming> {
        Some(raw.to_vec())
    }

    fn serialize(&self, msg: Self::Outgoing) -> Vec<u8> {
        msg
    }

    fn match_header(&self, header: &PvAccessHeader) -> bool {
        header.is_echo()
    }

    fn handle_message(
        &self,
        msg: &[u8],
        state: &mut ServerState,
    ) -> Result<ResponseMessage, anyhow::Error> {
        // Echo the message back
        Ok(ResponseMessage::Echo(msg.to_vec()))
    }
}
