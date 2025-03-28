use anyhow::{Result, anyhow};
use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
use std::io::{Cursor, Read, Write};

#[derive(Debug, Clone)]
pub struct SearchRequest {
    pub search_sequence_id: u32,
    pub flags: u8, // Bit 0: replyRequired, Bit 7: unicast/broadcast flag
    pub response_address: [u8; 16], // IPv6 address (or IPv4-mapped IPv6)
    pub response_port: u16,
    pub protocols: Vec<String>,
    pub channels: Vec<(u32, String)>, // (searchInstanceID, channelName)
}

impl SearchRequest {
    /// 🔹 Serialize `SearchRequest` to bytes
    pub fn to_bytes(&self) -> Result<Vec<u8>> {
        let mut buffer = Vec::new();
        buffer.write_u32::<BigEndian>(self.search_sequence_id)?;
        buffer.write_u8(self.flags)?;
        buffer.write_all(&[0; 3])?; // Reserved 3 bytes
        buffer.write_all(&self.response_address)?;
        buffer.write_u16::<BigEndian>(self.response_port)?;

        // Write protocols
        buffer.write_u8(self.protocols.len() as u8)?;
        for protocol in &self.protocols {
            buffer.write_u8(protocol.len() as u8)?;
            buffer.write_all(protocol.as_bytes())?;
        }

        // Write channels
        buffer.write_u16::<BigEndian>(self.channels.len() as u16)?;
        for (search_instance_id, channel_name) in &self.channels {
            buffer.write_u32::<BigEndian>(*search_instance_id)?;
            buffer.write_u8(channel_name.len() as u8)?;
            buffer.write_all(channel_name.as_bytes())?;
        }

        Ok(buffer)
    }

    /// 🔹 Deserialize `SearchRequest` from bytes
    pub fn from_bytes(bytes: &[u8]) -> Result<Self> {
        let mut cursor = Cursor::new(bytes);
        let search_sequence_id = cursor.read_u32::<BigEndian>()?;
        let flags = cursor.read_u8()?;
        cursor.consume(3); // Skip reserved bytes

        let mut response_address = [0u8; 16];
        cursor.read_exact(&mut response_address)?;
        let response_port = cursor.read_u16::<BigEndian>()?;

        // Read protocols
        let num_protocols = cursor.read_u8()? as usize;
        let mut protocols = Vec::with_capacity(num_protocols);
        for _ in 0..num_protocols {
            let len = cursor.read_u8()? as usize;
            let mut protocol_bytes = vec![0; len];
            cursor.read_exact(&mut protocol_bytes)?;
            protocols.push(String::from_utf8(protocol_bytes)?);
        }

        // Read channels
        let num_channels = cursor.read_u16::<BigEndian>()? as usize;
        let mut channels = Vec::with_capacity(num_channels);
        for _ in 0..num_channels {
            let search_instance_id = cursor.read_u32::<BigEndian>()?;
            let len = cursor.read_u8()? as usize;
            let mut channel_bytes = vec![0; len];
            cursor.read_exact(&mut channel_bytes)?;
            let channel_name = String::from_utf8(channel_bytes)?;
            channels.push((search_instance_id, channel_name));
        }

        Ok(Self {
            search_sequence_id,
            flags,
            response_address,
            response_port,
            protocols,
            channels,
        })
    }
}

#[derive(Debug, Clone)]
pub struct SearchResponse {
    pub guid: [u8; 12], // Server unique identifier
    pub search_sequence_id: u32,
    pub server_address: [u8; 16], // Server's IPv6 address (or IPv4-mapped IPv6)
    pub server_port: u16,
    pub protocol: String, // "tcp"
    pub found: bool,
    pub search_instance_ids: Vec<u32>, // IDs of found channels
}

impl SearchResponse {
    /// 🔹 Serialize `SearchResponse` to bytes
    pub fn to_bytes(&self) -> Result<Vec<u8>> {
        let mut buffer = Vec::new();
        buffer.write_all(&self.guid)?;
        buffer.write_u32::<BigEndian>(self.search_sequence_id)?;
        buffer.write_all(&self.server_address)?;
        buffer.write_u16::<BigEndian>(self.server_port)?;

        buffer.write_u8(self.protocol.len() as u8)?;
        buffer.write_all(self.protocol.as_bytes())?;

        buffer.write_u8(self.found as u8)?;

        buffer.write_u16::<BigEndian>(self.search_instance_ids.len() as u16)?;
        for id in &self.search_instance_ids {
            buffer.write_u32::<BigEndian>(*id)?;
        }

        Ok(buffer)
    }

    /// 🔹 Deserialize `SearchResponse` from bytes
    pub fn from_bytes(bytes: &[u8]) -> Result<Self> {
        let mut cursor = Cursor::new(bytes);
        let mut guid = [0u8; 12];
        cursor.read_exact(&mut guid)?;

        let search_sequence_id = cursor.read_u32::<BigEndian>()?;
        let mut server_address = [0u8; 16];
        cursor.read_exact(&mut server_address)?;
        let server_port = cursor.read_u16::<BigEndian>()?;

        let protocol_len = cursor.read_u8()? as usize;
        let mut protocol_bytes = vec![0; protocol_len];
        cursor.read_exact(&mut protocol_bytes)?;
        let protocol = String::from_utf8(protocol_bytes)?;

        let found = cursor.read_u8()? != 0;

        let num_ids = cursor.read_u16::<BigEndian>()? as usize;
        let mut search_instance_ids = Vec::with_capacity(num_ids);
        for _ in 0..num_ids {
            search_instance_ids.push(cursor.read_u32::<BigEndian>()?);
        }

        Ok(Self {
            guid,
            search_sequence_id,
            server_address,
            server_port,
            protocol,
            found,
            search_instance_ids,
        })
    }
}
