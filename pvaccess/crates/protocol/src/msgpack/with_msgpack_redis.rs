use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::VecDeque;

use crate::msgpack::msgpack_msg_types::{ChatMessage, ConnectionValidationRequest, EchoMessage};


struct Header {
    version: u8,
    msg_type: u8,
    msg_size: u32,
}
// full length in bytes is 6

impl Header {
    pub fn new(version: u8, msg_type: u8, msg_size: u32) -> Self {
        Header {
            version,
            msg_type,
            msg_size,
        }
    }
    pub fn from_bytes(bytes: &[u8]) -> Self {
        let version = bytes[0];
        let msg_type = bytes[1];
        let msg_size = u32::from_be_bytes([bytes[2], bytes[3], bytes[4], bytes[5]]);
        Header {
            version,
            msg_type,
            msg_size,
        }
    }
}

pub struct WithMsgPackRedis;

impl WithMsgPackRedis {
    pub fn process_bytes(&self, bytes: Vec<u8>) -> Result<(), String> {
        // read first 6 bytes to get the header
        let header = Header::from_bytes(&bytes[0..6]);
        match header.msg_type {
            0 => {
                // Echo
                let msg = rmp_serde::from_slice::<EchoMessage>(&bytes[7..]).unwrap();
                println!("Echo length: {}", msg.random_bytes.len());
            }
            1 => {
                // ConnectionValidation
                let msg =
                    rmp_serde::from_slice::<ConnectionValidationRequest>(&bytes[7..]).unwrap();
                println!(
                    "ConnectionValidation server introspection registry max size: {}",
                    msg.server_introspection_registry_max_size
                );
            }
            2 => {
                // Chat
                let msg = rmp_serde::from_slice::<ChatMessage>(&bytes[7..]).unwrap();
                println!("Chat: {}", msg.content);
            }
            _ => {
                return Err("Unknown message type".to_string());
            }
        }
        Ok(())
    }
}
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ChannelMetadata {
    pub name: String,
    pub schema: Value, // JSON Schema
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Channel {
    pub metadata: ChannelMetadata,
    pub messages: VecDeque<Value>, // Stores validated messages
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ChannelRequest {
    pub action: String,         // "create", "publish", "metadata"
    pub name: String,           // Channel name
    pub schema: Option<Value>,  // Used for "create"
    pub message: Option<Value>, // Used for "publish"
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ChannelResponse {
    pub success: bool,
    pub message: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum MsgType {
    Echo,
    ConnectionValidation,
    Chat,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Msg {
    pub msg_type: MsgType,
    pub content: String,
}
