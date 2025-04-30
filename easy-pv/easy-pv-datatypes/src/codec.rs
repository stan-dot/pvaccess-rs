use bytes::{Buf, BytesMut};
use std::io;
use tracing::debug;
use tokio_util::codec::Decoder;

use crate::header::PvAccessHeader;

pub struct PvAccessDecoder;

impl Decoder for PvAccessDecoder {
    type Item = (PvAccessHeader, BytesMut);
    type Error = io::Error;

    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        debug!("Decoding PvAccess frame...");
        if src.len() < PvAccessHeader::LEN {
            return Ok(None);
        }

        debug!("Buffer length: {}", src.len());
        let header_bytes = &src[..PvAccessHeader::LEN];
        let header = PvAccessHeader::from_bytes(header_bytes)
            .map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "Invalid header"))?;

        debug!("Header: {:?}", header);
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
