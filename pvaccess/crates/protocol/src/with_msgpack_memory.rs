use bincode;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use async_trait::async_trait;
use schemars::schema_for;
use std::any::Any;
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::memory_channel::ChannelStore;
use crate::protocol::Protocol;

/// 🔹 Message Header (common for all messages)
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct MessageHeader {
    pub msg_type: MsgType,
    pub message_id: u32, // Unique ID per message
    pub timestamp: u64,  // Epoch timestamp (for ordering)
}

/// 🔹 Message Types
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub enum MsgType {
    UdpBeacon,
    TcpConnectionValidation,
    TcpEcho,
    TcpEchoResponse,
    ChannelSearch,
    ChannelSearchResponse,
    ChannelCrud,
    TcpMonitorChanges,
    GetChannelSchema,
}

/// 🔹 Message Content (different per type)
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub enum MessageContent {
    /// UDP Beacon - No extra content, just presence signal
    UdpBeacon,

    /// TCP Connection Validation
    TcpConnectionValidation {
        client_id: String,
    },

    /// TCP Echo & Response
    TcpEcho {
        data: Vec<u8>,
    },
    TcpEchoResponse {
        response_data: Vec<u8>,
    },

    /// Search for channels using a regex
    ChannelSearch {
        query: String,
    },
    ChannelSearchResponse {
        matches: Vec<String>,
    },

    /// Create, Delete, or List channels
    ChannelCrud {
        action: CrudAction,
        channel_name: String,
    },

    /// Monitor changes in a channel
    TcpMonitorChanges {
        channel_name: String,
    },

    /// Request for the schema of a channel
    GetChannelSchema {
        channel_name: String,
    },
}

/// 🔹 CRUD Actions for channels
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub enum CrudAction {
    Create,
    Delete,
    List,
}

/// 🔹 Full Message (Header + Content)
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct Message {
    pub header: MessageHeader,
    pub content: MessageContent,
}
pub struct WithMsgpackMemory {
    channels: Arc<Mutex<ChannelStore>>,
}

#[async_trait]
impl Protocol for WithMsgpackMemory {
    fn discover_message(&self) -> Vec<u8> {
        b"DISCOVER_X_MEMORY".to_vec()
    }

    fn parse_header(&self, data: &[u8]) -> Result<Box<dyn Any>, String> {
        // todo make this work
        // let header = Header::from_bytes(&bytes[0..6]);
        bincode::deserialize::<MessageHeader>(data)
            .map(|h| Box::new(h) as Box<dyn Any>)
            .map_err(|_| "Failed to parse header".to_string())
    }

    async fn create_channel(&self, name: &str) -> bool {
        let schema = serde_json::to_string(&schema_for!(Message)).unwrap();
        let mut store = self.channels.lock().await;
        store.create_channel(name, schema)
    }

    async fn delete_channel(&self, name: &str) -> bool {
        let mut store = self.channels.lock().await;
        store.delete_channel(name)
    }

    async fn list_channels(&self) -> Vec<String> {
        let store = self.channels.lock().await;
        store.channels.keys().cloned().collect()
    }

    async fn channel_put(&self, channel_name: &str, message: String) -> bool {
        let mut store = self.channels.lock().await;
        store.put_message(channel_name, message)
    }

    async fn channel_get(&self, channel_name: &str, limit: usize) -> Vec<String> {
        let store = self.channels.lock().await;
        store.get_messages(channel_name, limit)
    }
}
