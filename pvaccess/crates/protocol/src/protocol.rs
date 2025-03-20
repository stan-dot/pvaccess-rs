use std::any::Any;

pub trait Protocol {
    /// 🔹 Generate a UDP discovery message
    fn discover_message(&self) -> Vec<u8>;

    /// 🔹 Parse a message header (returns `Box<dyn Any>` since headers differ)
    fn parse_header(&self, data: &[u8]) -> Result<Box<dyn Any>, String>;

    /// 🔹 Create a new channel
    fn create_channel(&mut self, name: &str) -> bool;

    /// 🔹 Delete a channel
    fn delete_channel(&mut self, name: &str) -> bool;

    /// 🔹 List all active channels
    fn list_channels(&self) -> Vec<String>;
}
