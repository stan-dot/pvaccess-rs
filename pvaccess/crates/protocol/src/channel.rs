use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::VecDeque;

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
