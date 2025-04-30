use byteorder::{BigEndian, LittleEndian, ReadBytesExt};
use std::{
    fmt::format,
    io::{Cursor, Result},
};

use crate::messages::into::ToBytes;

pub const HEADER_LENGTH: usize = 8; // Header length in bytes

/// 🔹 `pvAccess` Protocol Header (fixed 8-byte structure)
#[derive(Debug, Clone, Copy)]
pub struct PvAccessHeader {
    pub magic: u8,                // Always 0xCA
    pub version: u8,              // Protocol version
    pub flags: u8,                // Bitmask flags (endianness, segmentation, etc.)
    pub message_command: Command, // Message type
    pub payload_size: u32,        // Length of payload (non-aligned bytes)
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Command {
    Beacon = 0x00,
    ConnectionValidation = 0x01,
    Echo = 0x03,
    Unknown = 0xFF,
}

impl From<u8> for Command {
    fn from(byte: u8) -> Self {
        match byte {
            0x00 => Command::Beacon,
            0x01 => Command::ConnectionValidation,
            0x03 => Command::Echo,
            _ => Command::Unknown,
        }
    }
}

// https://docs.epics-controls.org/en/latest/pv-access/protocol.html#version-2
// on version
impl PvAccessHeader {
    pub const LEN: usize = 8; // Header length in bytes
    /// 🔹 Create a new header
    pub fn new(flags: u8, message_command: Command, payload_size: u32) -> Self {
        Self {
            magic: 0xCA,
            version: 2,
            flags,
            message_command,
            payload_size,
        }
    }

    /// 🔹 Deserialize from bytes
    pub fn from_bytes(bytes: &[u8]) -> Result<Self> {
        if bytes.len() < 8 {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "Header too short",
            ));
        }

        let mut cursor = Cursor::new(bytes);
        let magic = cursor.read_u8()?;
        if magic != 0xCA {
            let error_text = format!(
                "Invalid magic byte in bytes {:?}",
                bytes.to_ascii_lowercase()
            );
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                error_text,
            ));
        }

        let version = cursor.read_u8()?;
        let flags = cursor.read_u8()?;
        let message_command = Command::from(cursor.read_u8()?);

        let payload_size = if flags & 0b1000_0000 != 0 {
            cursor.read_u32::<BigEndian>()?
        } else {
            cursor.read_u32::<LittleEndian>()?
        };

        Ok(Self {
            magic,
            version,
            flags,
            message_command,
            payload_size,
        })
    }

    /// 🔹 Check if message is segmented
    pub fn is_segmented(&self) -> bool {
        matches!(
            self.flags & 0b0011_0000,
            0b0001_0000 | 0b0010_0000 | 0b0011_0000
        )
    }

    /// 🔹 Check if message is from server
    pub fn is_server_message(&self) -> bool {
        self.flags & 0b0100_0000 != 0
    }

    /// 🔹 Check endianness
    pub fn is_big_endian(&self) -> bool {
        self.flags & 0b1000_0000 != 0
    }
}

impl ToBytes for PvAccessHeader {
    /// 🔹 Serialize to bytes
    fn to_bytes(&self) -> anyhow::Result<Vec<u8>> {
        {
            use byteorder::WriteBytesExt;
            let mut buffer = Vec::new();
            buffer.write_u8(self.magic)?;
            buffer.write_u8(self.version)?;
            buffer.write_u8(self.flags)?;
            buffer.write_u8(self.message_command as u8)?;

            if self.flags & 0b1000_0000 != 0 {
                buffer.write_u32::<BigEndian>(self.payload_size)?;
            } else {
                buffer.write_u32::<LittleEndian>(self.payload_size)?;
            }
            Ok(buffer)
        }
    }
}

#[test]
fn test_header_serialization() {
    let header = PvAccessHeader::new(0b0100_0000, Command::Beacon, 1234); // Server message, command 5, payload 1234
    let bytes = header.to_bytes().unwrap();
    let parsed_header = PvAccessHeader::from_bytes(&bytes).unwrap();
    assert_eq!(header.magic, parsed_header.magic);
    assert_eq!(header.version, parsed_header.version);
    assert_eq!(header.flags, parsed_header.flags);
    assert_eq!(header.message_command, parsed_header.message_command);
    assert_eq!(header.payload_size, parsed_header.payload_size);
}

#[test]
fn test_from_bytes_correct() {
    let bytes = [
        202, 2, 0, 0, 27, 0, 0, 0, 247, 42, 160, 206, 226, 127, 65, 190, 187, 51, 137, 1, 0, 2, 0,
        0, 127, 0, 0, 1, 21, 200, 3, 116, 99, 112, 0,
    ];
    println!("{:?}", bytes);
    let h = PvAccessHeader::from_bytes(&bytes[..PvAccessHeader::LEN]).unwrap();
    println!("{:?}", h);
    assert_eq!(h.magic, 0xCA)
}
