pub trait IntoPvAccessFrame {
    fn into_frame(self, command: Command, flags: u8) -> Result<PvAccessFrame>;
}

use crate::protocol::{Command, PvAccessFrame, PvAccessHeader};
use bytes::Bytes;

impl<T> IntoPvAccessFrame for T
where
    T: ToBytes, // your trait to serialize into bytes
{
    fn into_frame(self, command: Command, flags: u8) -> Result<PvAccessFrame> {
        let payload = self.to_bytes()?;
        let header = PvAccessHeader::new(flags, command, payload.len() as u32);
        Ok(PvAccessFrame {
            header,
            payload: Bytes::from(payload),
        })
    }
}
