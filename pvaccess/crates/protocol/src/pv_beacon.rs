use byteorder::{BigEndian, WriteBytesExt};
use std::io::{Cursor, Result};
use uuid::Uuid;

/// 🔹 UDP Beacon Message (Sent with Command `0x01`)
#[derive(Debug, Clone)]
pub struct BeaconMessage {
    pub guid: [u8; 12],           // Server GUID (MUST change every restart)
    pub flags: u8,                // Reserved (set to 0)
    pub beacon_sequence_id: u8,   // Counter with rollover
    pub change_count: u16,        // Increments when channels change
    pub server_address: [u8; 16], // IPv6 address (or IPv4 encoded in IPv6)
    pub server_port: u16,         // Port where the server is listening
    pub protocol: String,         // Protocol name ("tcp")
    pub server_status_if: u8,     // NULL_TYPE_CODE if no status
}

impl BeaconMessage {
    /// 🔹 Create a new beacon message
    pub fn new(server_port: u16) -> Self {
        let guid = Uuid::new_v4(); // Generate new GUID for each restart
        let mut server_address = [0u8; 16];

        // Set IPv4 address as encoded IPv6 (e.g., "::FFFF:192.168.1.1")
        let ipv4 = [192, 168, 1, 100]; // todo Replace with real address read from Kubernetes
        server_address[10] = 0xFF;
        server_address[11] = 0xFF;
        server_address[12..16].copy_from_slice(&ipv4);

        Self {
            guid: guid.as_bytes()[..12].try_into().unwrap(),
            flags: 0,
            beacon_sequence_id: 0,
            change_count: 0,
            server_address,
            server_port,
            protocol: "tcp".into(),
            server_status_if: 0, // NULL_TYPE_CODE
        }
    }

    /// 🔹 Serialize to bytes
    pub fn to_bytes(&self) -> Result<Vec<u8>> {
        let mut buffer = Vec::new();

        buffer.extend_from_slice(&self.guid);
        buffer.write_u8(self.flags)?;
        buffer.write_u8(self.beacon_sequence_id)?;
        buffer.write_u16::<BigEndian>(self.change_count)?;
        buffer.extend_from_slice(&self.server_address);
        buffer.write_u16::<BigEndian>(self.server_port)?;

        buffer.write_u8(self.protocol.len() as u8)?;
        buffer.extend_from_slice(self.protocol.as_bytes());

        buffer.write_u8(self.server_status_if)?;

        Ok(buffer)
    }
}
