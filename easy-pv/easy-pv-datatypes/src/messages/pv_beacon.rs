use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
use std::env;
use std::fmt;
use std::io::Read;
use std::io::{Cursor, Result};
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};
use tracing::debug;
use uuid::Uuid;

use super::into::ToBytes;

/// 🔹 UDP Beacon Message (Sent with Command `0x01`)
#[derive(Debug, Clone)]
pub struct BeaconMessage {
    pub guid: [u8; 12],         // Server GUID (MUST change every restart)
    pub flags: u8,              // Reserved (set to 0)
    pub beacon_sequence_id: u8, // Counter with rollover
    pub change_count: u16,      // Increments when channels change
    pub server_address: IpAddr, // IPv6 address (or IPv4 encoded in IPv6)
    pub server_port: u16,       // Port where the server is listening
    pub protocol: String,       // Protocol name ("tcp")
    pub server_status_if: u8,   // NULL_TYPE_CODE if no status
}

impl fmt::Display for BeaconMessage {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Beacon with sequence id: {} and flags {}",
            self.beacon_sequence_id, self.flags
        )
    }
}

fn parse_ip(bytes: &[u8; 16]) -> IpAddr {
    // Check if it's an IPv4-mapped IPv6 address
    if bytes[..12] == [0; 10].as_ref().repeat(1).as_slice()[..]
        && bytes[10] == 0xFF
        && bytes[11] == 0xFF
    {
        IpAddr::V4(Ipv4Addr::new(bytes[12], bytes[13], bytes[14], bytes[15]))
    } else {
        IpAddr::V6(Ipv6Addr::from(*bytes))
    }
}

impl BeaconMessage {
    /// 🔹 Create a new beacon message
    pub fn new(server_port: u16, server_uid: Uuid) -> Self {
        let mut server_address = [0u8; 16];

        // Get IPv4 address from env and parse it
        let ipv4: Ipv4Addr = env::var("SERVER_IP")
            .expect("SERVER_IP not set")
            .parse()
            .expect("Invalid IPv4 address");

        server_address[10] = 0xFF;
        server_address[11] = 0xFF;
        // Set IPv4 address as encoded IPv6 (e.g., "::FFFF:192.168.1.1")
        server_address[12..16].copy_from_slice(&ipv4.octets());

        Self {
            guid: server_uid.as_bytes()[..12].try_into().unwrap(),
            flags: 0,
            beacon_sequence_id: 0,
            change_count: 0,
            server_address: parse_ip(&server_address),
            server_port,
            protocol: "tcp".into(),
            server_status_if: 0, // NULL_TYPE_CODE
        }
    }

    pub fn from_bytes(bytes: &[u8]) -> Result<Self> {
        let mut cursor = Cursor::new(bytes);

        let mut guid = [0u8; 12];
        let _ = cursor.read_exact(&mut guid);

        let flags = cursor.read_u8()?;
        let beacon_sequence_id = cursor.read_u8()?;
        let change_count = cursor.read_u16::<BigEndian>()?;
        let mut server_address_buffer = [0u8; 16];
        let _ = cursor.read_exact(&mut server_address_buffer)?;
        let addr: IpAddr = parse_ip(&server_address_buffer);
        let server_port = cursor.read_u16::<BigEndian>()?;

        let protocol_length = cursor.read_u8()?;
        let mut protocol_bytes = vec![0u8; protocol_length as usize];
        let _ = cursor.read_exact(&mut protocol_bytes);
        let protocol = String::from_utf8(protocol_bytes)
            .map_err(|_| std::io::Error::new(std::io::ErrorKind::InvalidData, "Invalid UTF-8"))?;

        let server_status_if = cursor.read_u8()?;

        Ok(Self {
            guid,
            flags,
            beacon_sequence_id,
            change_count,
            server_address: addr,
            server_port,
            protocol,
            server_status_if,
        })
    }
}

impl ToBytes for BeaconMessage {
    /// 🔹 Serialize to bytes
    fn to_bytes(&self) -> anyhow::Result<Vec<u8>> {
        debug!("self: {:?}", self);
        let mut buffer = Vec::new();

        buffer.extend_from_slice(&self.guid);
        buffer.write_u8(self.flags)?;
        buffer.write_u8(self.beacon_sequence_id)?;
        buffer.write_u16::<BigEndian>(self.change_count)?;

        let slice = match self.server_address {
            IpAddr::V4(ipv4) => &ipv4.to_ipv6_mapped().octets(),
            IpAddr::V6(ipv6) => &ipv6.octets(),
        };
        buffer.extend_from_slice(slice);
        //
        buffer.write_u16::<BigEndian>(self.server_port)?;

        buffer.write_u8(self.protocol.len() as u8)?;
        buffer.extend_from_slice(self.protocol.as_bytes());

        buffer.write_u8(self.server_status_if)?;
        // println!("new beacon body Buffer length: {}", buffer.len());

        Ok(buffer)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_crease_body_correctly() {
        let n = BeaconMessage {
            guid: [87, 186, 234, 203, 160, 226, 76, 60, 157, 167, 71, 104],
            flags: 0,
            beacon_sequence_id: 1,
            change_count: 0,
            server_address: IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)),
            server_port: 5576,
            protocol: "tcp".to_string(),
            server_status_if: 0,
        };
        let bytes = n.to_bytes().unwrap();
        println!("{:?}", bytes);
        let beacon = BeaconMessage::from_bytes(&bytes).unwrap();
        assert_eq!(beacon.flags, 0);
        assert_eq!(beacon.beacon_sequence_id, 1);
    }
}
