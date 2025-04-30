use bytes::Bytes;

use crate::{
    frame::PvAccessFrame,
    header::{Command, PvAccessHeader},
};

pub trait ToBytes {
    fn to_bytes(&self) -> anyhow::Result<Vec<u8>>;
}

pub trait IntoPvAccessFrame {
    fn into_frame(self, command: Command, flags: u8) -> Result<PvAccessFrame, anyhow::Error>;
}

impl<T> IntoPvAccessFrame for T
where
    T: ToBytes, // your trait to serialize into bytes
{
    fn into_frame(self, command: Command, flags: u8) -> Result<PvAccessFrame, anyhow::Error> {
        let payload = self.to_bytes()?;
        let header = PvAccessHeader::new(flags, command, payload.len() as u32);
        Ok(PvAccessFrame {
            header,
            payload: Bytes::from(payload),
        })
    }

}
