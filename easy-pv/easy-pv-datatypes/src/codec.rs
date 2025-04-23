use bytes::{Buf, BytesMut};
use futures::StreamExt;
use std::io;
use std::sync::{Arc, Mutex};
use tokio_util::codec::{Decoder, FramedRead};

use crate::header::PvAccessHeader;

pub struct PvAccessDecoder;

impl Decoder for PvAccessDecoder {
    type Item = (PvAccessHeader, BytesMut);
    type Error = io::Error;

    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        // Need enough for header?
        if src.len() < PvAccessHeader::LEN {
            return Ok(None);
        }

        let header_bytes = &src[..PvAccessHeader::LEN];
        let header = PvAccessHeader::from_bytes(header_bytes)
            .map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "Invalid header"))?;

        let total_len = PvAccessHeader::LEN + header.payload_size as usize;
        if src.len() < total_len {
            return Ok(None); // Not enough data yet
        }

        // Advance past header
        src.advance(PvAccessHeader::LEN);
        let payload = src.split_to(header.payload_size as usize);

        Ok(Some((header, payload)))
    }
}
