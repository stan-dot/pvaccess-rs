use bytes::{BufMut, Bytes, BytesMut};
use std::io;
use tokio_util::codec::{Decoder, Encoder};

use crate::header::PvAccessHeader;

#[derive(Debug)]
pub struct PvAccessFrame {
    pub header: PvAccessHeader,
    pub payload: Bytes, // todo might consider the payload a union of non-binary types
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

impl Decoder for PvAccessEncoder {
    type Error = io::Error;

    type Item = PvAccessFrame;

    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        // todo first read out the bytes from the pv header
        // then read out the bytes from the body
        todo!()
    }
}
