use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
use std::io::{Cursor, Result};
use tokio::io::AsyncReadExt;

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
            let mut auth_bytes = vec![0; len];
            cursor.read_exact(&mut auth_bytes);
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
    pub connection_qos: u16, // Quality of Service flags
    pub auth_nz: String,     // Selected authentication plugin
}

impl ConnectionValidationResponse {
    /// 🔹 Serialize to bytes
    pub fn to_bytes(&self) -> Result<Vec<u8>> {
        let mut buffer = Vec::new();
        buffer.write_u32::<BigEndian>(self.client_receive_buffer_size)?;
        buffer.write_u16::<BigEndian>(self.client_introspection_registry_max_size)?;
        buffer.write_u16::<BigEndian>(self.connection_qos)?;

        buffer.write_u8(self.auth_nz.len() as u8)?;
        buffer.extend_from_slice(self.auth_nz.as_bytes());

        Ok(buffer)
    }

    /// 🔹 Deserialize from bytes
    pub fn from_bytes(bytes: &[u8]) -> Result<Self> {
        let mut cursor = Cursor::new(bytes);
        let client_receive_buffer_size = cursor.read_u32::<BigEndian>()?;
        let client_introspection_registry_max_size = cursor.read_u16::<BigEndian>()?;
        let connection_qos = cursor.read_u16::<BigEndian>()?;

        let len = cursor.read_u8()? as usize;
        let mut auth_bytes = vec![0; len];
        cursor.read_exact(&mut auth_bytes)?;
        let auth_nz = String::from_utf8(auth_bytes).unwrap();

        Ok(Self {
            client_receive_buffer_size,
            client_introspection_registry_max_size,
            connection_qos,
            auth_nz,
        })
    }
}
