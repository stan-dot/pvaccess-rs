use async_trait::async_trait;
use bincode::config::Endianness;
use easy_pv_datatypes::header::PvAccessHeader;
use std::sync::MutexGuard;

use crate::state::ServerState;

#[async_trait]
pub trait Feature: Send + Sync {
    fn name(&self) -> &'static str;
    fn match_header(&self, header: &PvAccessHeader) -> bool;

    type Incoming: Send;
    type Outgoing: Send;

    async fn handle_message<'a>(
        &self,
        msg: Self::Incoming,
        state: &mut MutexGuard<'a, ServerState>,
    ) -> Result<Self::Outgoing, anyhow::Error>;

    /// How to parse raw bytes into `Self::Incoming`
    fn parse(&self, raw: &[u8], endianness: Endianness) -> Option<Self::Incoming>;

    /// How to serialize `Self::Outgoing` to bytes
    fn serialize(&self, msg: Self::Outgoing) -> Vec<u8>;
}


// NOTE this is an abstract impl that would work as a blanket for all such traits - so they have a default impl
// I need the features unique anyway, so atm no need for a blanket impl
// impl<T> Feature for T
// where
//     T: HandlesMessages + 'static,
// {
//     fn name(&self) -> &'static str {
//         self.name()
//     }

//     fn match_header(&self, header: &PvAccessHeader) -> bool {
//         self.match_header(header)
//     }

//     fn handle_message(
//         &self,
//         msg: &[u8],
//         state: &mut ServerState,
//     ) -> Result<ResponseMessage, Error> {
//         let parsed = T::Incoming::try_from(msg)?;
//         let response = self.handle(parsed, state)?;
//         Ok(response.into())
//     }
// }