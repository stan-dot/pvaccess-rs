use byteorder::{BigEndian, LittleEndian, ReadBytesExt, WriteBytesExt};
use std::io::Read;
use std::io::{Cursor, Result};

use super::into::ToBytes;

/// 🔹 Echo Message (Sent by Client)
#[derive(Debug, Clone)]
pub struct EchoMessage {
    pub random_bytes: Vec<u8>, // Payload
}

impl EchoMessage {
    /// 🔹 Serialize to bytes
    pub fn to_bytes(&self, is_big_endian: bool) -> Result<Vec<u8>> {
        let mut buffer = Vec::new();

        if is_big_endian {
            buffer.write_u16::<BigEndian>(self.random_bytes.len() as u16)?;
        } else {
            buffer.write_u16::<LittleEndian>(self.random_bytes.len() as u16)?;
        }

        buffer.extend_from_slice(&self.random_bytes);

        Ok(buffer)
    }

    /// 🔹 Deserialize from bytes
    pub fn from_bytes(bytes: &[u8], is_big_endian: bool) -> Result<Self> {
        let mut cursor = Cursor::new(bytes);

        let length = if is_big_endian {
            cursor.read_u16::<BigEndian>()?
        } else {
            cursor.read_u16::<LittleEndian>()?
        } as usize;

        if bytes.len() < 2 + length {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "Invalid Echo message",
            ));
        }

        let mut payload = vec![0; length];
        cursor.read_exact(&mut payload)?;

        Ok(Self {
            random_bytes: payload,
        })
    }
}

impl ToBytes for EchoMessage {
    /// 🔹 Serialize to bytes
    fn to_bytes(&self) -> anyhow::Result<Vec<u8>> {
        let mut buffer = Vec::new();
        buffer.write_u16::<BigEndian>(self.random_bytes.len() as u16)?;
        buffer.extend_from_slice(&self.random_bytes);
        Ok(buffer)
    }
}

/// 🔹 Echo Response Message (Sent by Server)
#[derive(Debug, Clone)]
pub struct EchoResponse {
    pub repeated_bytes: Vec<u8>,
}

impl EchoResponse {
    /// 🔹 Serialize to bytes
    pub fn to_bytes(&self, is_big_endian: bool) -> Result<Vec<u8>> {
        let mut buffer = Vec::new();

        buffer.extend_from_slice(&self.repeated_bytes);

        Ok(buffer)
    }

    /// 🔹 Deserialize from bytes
    pub fn from_bytes(bytes: &[u8], is_big_endian: bool) -> Result<Self> {
        let mut cursor = Cursor::new(bytes);

        let length = if is_big_endian {
            cursor.read_u16::<BigEndian>()?
        } else {
            cursor.read_u16::<LittleEndian>()?
        } as usize;

        if bytes.len() < 2 + length {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "Invalid Echo response",
            ));
        }

        let mut payload = vec![0; length];
        cursor.read_exact(&mut payload)?;

        Ok(Self {
            repeated_bytes: payload,
        })
    }
}

impl ToBytes for EchoResponse {
    fn to_bytes(&self) -> anyhow::Result<Vec<u8>> {
        let mut buffer = Vec::new();
        buffer.write_u16::<BigEndian>(self.repeated_bytes.len() as u16)?;
        // if is_big_endian {
        //     buffer.write_u16::<BigEndian>(self.repeated_bytes.len() as u16)?;
        // } else {
        //     buffer.write_u16::<LittleEndian>(self.repeated_bytes.len() as u16)?;
        // }
        buffer.extend_from_slice(&self.repeated_bytes);
        Ok(buffer)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_echo_message_serialization() {
        let message = EchoMessage {
            random_bytes: vec![1, 2, 3, 4, 5],
        };

        let bytes_le = message.to_bytes(false).unwrap();
        let parsed_le = EchoMessage::from_bytes(&bytes_le, false).unwrap();
        assert_eq!(message.random_bytes, parsed_le.random_bytes);

        let bytes_be = message.to_bytes(true).unwrap();
        let parsed_be = EchoMessage::from_bytes(&bytes_be, true).unwrap();
        assert_eq!(message.random_bytes, parsed_be.random_bytes);
    }

    #[test]
    fn test_echo_response_serialization() {
        let response = EchoResponse {
            repeated_bytes: vec![9, 8, 7, 6, 5],
        };

        let bytes_le = response.to_bytes(false).unwrap();
        let parsed_le = EchoResponse::from_bytes(&bytes_le, false).unwrap();
        assert_eq!(response.repeated_bytes, parsed_le.repeated_bytes);

        let bytes_be = response.to_bytes(true).unwrap();
        let parsed_be = EchoResponse::from_bytes(&bytes_be, true).unwrap();
        assert_eq!(response.repeated_bytes, parsed_be.repeated_bytes);
    }
}
