use anyhow::anyhow;
use bitflags::bitflags;
use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
// use std::io::{Cursor, Read, Result, Write};
use std::io::{Cursor, Result};
// use tokio::io::AsyncReadExt;

/// 🔹 Connection Validation Request (Sent by Server)
#[derive(Debug, Clone)]
pub struct ConnectionValidationRequest {
    pub server_receive_buffer_size: u32,
    pub server_introspection_registry_max_size: u16,
    pub auth_nz: Vec<String>, // Supported authentication mechanisms
}

impl ConnectionValidationRequest {
    /// 🔹 Serialize to bytes
    pub fn to_bytes(&self) -> Result<Vec<u8>> {
        let mut buffer = Vec::new();
        buffer.write_u32::<BigEndian>(self.server_receive_buffer_size)?;
        buffer.write_u16::<BigEndian>(self.server_introspection_registry_max_size)?;

        // Write authentication list
        buffer.write_u8(self.auth_nz.len() as u8)?;
        for auth in &self.auth_nz {
            buffer.write_u8(auth.len() as u8)?;
            buffer.extend_from_slice(auth.as_bytes());
        }

        Ok(buffer)
    }

    /// 🔹 Deserialize from bytes
    pub fn from_bytes(bytes: &[u8]) -> Result<Self> {
        let mut cursor = Cursor::new(bytes);
        let server_receive_buffer_size = cursor.read_u32::<BigEndian>()?;
        let server_introspection_registry_max_size = cursor.read_u16::<BigEndian>()?;

        // todo make dependent on the header flag
        let auth_count = cursor.read_u8()? as usize;
        let mut auth_nz = Vec::new();
        for _ in 0..auth_count {
            let len = cursor.read_u8().unwrap();
            let mut auth_bytes = vec![0; len.into()];
            {
                use std::io::Read;
                cursor.read_exact(&mut auth_bytes);
            }
            auth_nz.push(String::from_utf8(auth_bytes).unwrap());
        }

        Ok(Self {
            server_receive_buffer_size,
            server_introspection_registry_max_size,
            auth_nz,
        })
    }
}

/// 🔹 Connection Validation Response (Sent by Client)
#[derive(Debug, Clone)]
pub struct ConnectionValidationResponse {
    pub client_receive_buffer_size: u32,
    pub client_introspection_registry_max_size: u16,
    pub connection_qos: ConnectionQoS, // 🔹 QoS is now a bitflag enum
    pub auth_nz: String,               // Selected authentication plugin
}

impl ConnectionValidationResponse {
    /// 🔹 Serialize to bytes
    pub fn to_bytes(&self) -> Result<Vec<u8>> {
        let mut buffer = Vec::new();
        buffer.write_u32::<BigEndian>(self.client_receive_buffer_size)?;
        buffer.write_u16::<BigEndian>(self.client_introspection_registry_max_size)?;
        buffer.write_u16::<BigEndian>(self.connection_qos.bits())?;

        buffer.write_u8(self.auth_nz.len() as u8)?;
        buffer.extend_from_slice(self.auth_nz.as_bytes());

        Ok(buffer)
    }

    /// 🔹 Deserialize from bytes
    pub fn from_bytes(bytes: &[u8]) -> Result<Self> {
        let mut cursor = Cursor::new(bytes);
        let client_receive_buffer_size = cursor.read_u32::<BigEndian>()?;
        let client_introspection_registry_max_size = cursor.read_u16::<BigEndian>()?;
        let connection_qos_bits = cursor.read_u16::<BigEndian>()?;

        let connection_qos = ConnectionQoS::from_bits(connection_qos_bits)
            .ok_or(anyhow!("Invalid QoS flags: {:#b}", connection_qos_bits))
            .unwrap();

        let len = cursor.read_u8()? as usize;
        let mut auth_bytes = vec![0; len];
        {
            use std::io::Read;
            cursor.read_exact(&mut auth_bytes)?;
        }
        let auth_nz = String::from_utf8(auth_bytes).unwrap();

        Ok(Self {
            client_receive_buffer_size,
            client_introspection_registry_max_size,
            connection_qos,
            auth_nz,
        })
    }
}

bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct ConnectionQoS: u16 {
        const PRIORITY_MASK      = 0b0000_0000_0111_1111;  // Bits 0-6 (0-100 priority level)
        const LOW_LATENCY        = 0b0000_0001_0000_0000;  // Bit 8
        const THROUGHPUT        = 0b0000_0010_0000_0000;  // Bit 9
        const ENABLE_COMPRESSION = 0b0000_0100_0000_0000;  // Bit 10
    }
}

impl ConnectionQoS {
    /// 🔹 Extracts the **priority level** (0-100) from the QoS bits.
    pub fn priority_level(self) -> u8 {
        (self.bits() & Self::PRIORITY_MASK.bits()) as u8
    }

    /// 🔹 Constructs a QoS with a specific priority (0-100)
    pub fn with_priority(priority: u8) -> Self {
        let mut flags = Self::empty();
        flags.insert(Self::from_bits_truncate(priority as u16));
        flags
    }
}
