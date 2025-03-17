use std::collections::HashMap;
use std::collections::VecDeque;
use tokio::sync::RwLock;
use serde_json::Value;
use jsonschema::{JSONSchema, CompilationError};
use shared::{Channel, ChannelRequest, ChannelResponse, ChannelMetadata};
use std::sync::Arc;

#[derive(Clone)]
struct ServerState {
    channels: Arc<RwLock<HashMap<String, Channel>>>,
}

impl ServerState {
    fn new() -> Self {
        Self {
            channels: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    async fn handle_channel_request(&self, request: ChannelRequest) -> ChannelResponse {
        let mut channels = self.channels.write().await;

        match request.action.as_str() {
            "create" => {
                if channels.contains_key(&request.name) {
                    return ChannelResponse { success: false, message: "Channel already exists".into() };
                }

                if let Some(schema) = &request.schema {
                    // Validate if schema itself is correct
                    if JSONSchema::compile(schema).is_err() {
                        return ChannelResponse { success: false, message: "Invalid JSON Schema".into() };
                    }

                    let channel = Channel {
                        metadata: ChannelMetadata {
                            name: request.name.clone(),
                            schema: schema.clone(),
                        },
                        messages: VecDeque::with_capacity(100), // Limited message buffer
                    };

                    channels.insert(request.name.clone(), channel);
                    return ChannelResponse { success: true, message: "Channel created successfully".into() };
                } else {
                    return ChannelResponse { success: false, message: "Missing schema".into() };
                }
            }
            "publish" => {
                if let Some(channel) = channels.get_mut(&request.name) {
                    if let Some(message) = &request.message {
                        let schema = JSONSchema::compile(&channel.metadata.schema).unwrap();
                        if schema.is_valid(message) {
                            if channel.messages.len() >= 100 {
                                channel.messages.pop_front(); // Keep message buffer limited
                            }
                            channel.messages.push_back(message.clone());

                            return ChannelResponse { success: true, message: "Message accepted".into() };
                        } else {
                            return ChannelResponse { success: false, message: "Message failed schema validation".into() };
                        }
                    } else {
                        return ChannelResponse { success: false, message: "No message provided".into() };
                    }
                } else {
                    return ChannelResponse { success: false, message: "Channel not found".into() };
                }
            }
            "metadata" => {
                if let Some(channel) = channels.get(&request.name) {
                    let metadata_str = format!("{:?}", channel.metadata);
                    return ChannelResponse { success: true, message: metadata_str };
                } else {
                    return ChannelResponse { success: false, message: "Channel not found".into() };
                }
            }
            _ => ChannelResponse { success: false, message: "Invalid action".into() },
        }
    }
}
