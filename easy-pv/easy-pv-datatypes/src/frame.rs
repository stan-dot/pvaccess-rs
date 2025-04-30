use bytes::{BufMut, Bytes, BytesMut};
use std::io;
use tokio_util::codec::{Decoder, Encoder};

use crate::header::PvAccessHeader;

#[derive(Debug)]
pub struct PvAccessFrame {
    pub header: PvAccessHeader,
    pub payload: Bytes,
}
pub struct PvAccessEncoder;

impl Encoder<PvAccessFrame> for PvAccessEncoder {
    type Error = io::Error;

    fn encode(&mut self, item: PvAccessFrame, dst: &mut BytesMut) -> Result<(), Self::Error> {
        dst.put_u8(item.header.magic);
        dst.put_u8(item.header.version);
        dst.put_u8(item.header.flags.bits());
        dst.put_u8(item.header.message_command as u8);
        dst.put_u32(item.header.payload_size);

        dst.extend_from_slice(&item.payload);
        Ok(())
    }
}
