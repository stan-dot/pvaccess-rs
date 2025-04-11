use anyhow::Error;
use async_trait::async_trait;
use std::any::Any;

#[async_trait]
pub trait ProtocolServer: Send + Sync {
    type Header: Any + Send + Sync;
    /// 🔹 Generate a UDP discovery message
    fn discover_message(&self) -> Vec<u8>;

    /// 🔹 Parse a message header (returns `Box<dyn Any>` since headers differ)
    fn parse_header(&self, data: &[u8]) -> Result<Self::Header, Error>;

    /// 🔹 Create a new channel
    async fn create_channel(&self, name: &str) -> bool;

    /// 🔹 Delete a channel
    async fn delete_channel(&self, name: &str) -> bool;

    /// 🔹 List all active channels
    async fn list_channels(&self) -> Vec<String>;

    /// 🔹 Add a message to a channel
    async fn channel_put(&self, channel_name: &str, message: String) -> bool;

    /// 🔹 Retrieve messages from a channel
    async fn channel_get(&self, channel_name: &str, limit: usize) -> Vec<String>;
}
