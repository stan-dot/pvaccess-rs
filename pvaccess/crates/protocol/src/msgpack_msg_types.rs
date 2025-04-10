use serde::Deserialize;

/// 🔹 Echo Message (Sent by Client)
#[derive(Debug, Clone, Deserialize)]
pub struct EchoMessage {
    pub random_bytes: Vec<u8>, // Payload
}

#[derive(Debug, Clone, Deserialize)]
pub struct ConnectionValidationRequest {
    pub server_receive_buffer_size: i32,
    pub server_introspection_registry_max_size: i16,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ChatMessage {
    pub content: String,
}
