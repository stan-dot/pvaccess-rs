use regex::RegexSet;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};

/// 🔹 Represents a Channel (holds subscribers & schema)
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct Channel {
    pub name: String,
    pub subscribers: Vec<String>,   // List of client IDs
    pub schema: String,             // JSON Schema string
    pub messages: VecDeque<String>, // Messages (FIFO queue)
}

/// 🔹 In-Memory Channel Store
#[derive(Debug)]
pub struct ChannelStore {
    pub channels: HashMap<String, Channel>,
}

impl ChannelStore {
    /// 🔹 Create a new store
    pub fn new() -> Self {
        Self {
            channels: HashMap::new(),
        }
    }

    /// 🔹 Create a new channel
    pub fn create_channel(&mut self, name: &str, schema: String) -> bool {
        if self.channels.contains_key(name) {
            return false; // Channel already exists
        }
        self.channels.insert(
            name.to_string(),
            Channel {
                name: name.to_string(),
                subscribers: vec![],
                schema,
                            messages: VecDeque::new(),
            },
        );
        true
    }

    /// 🔹 Delete a channel
    pub fn delete_channel(&mut self, name: &str) -> bool {
        self.channels.remove(name).is_some()
    }

    /// 🔹 List all channels matching a regex pattern
    pub fn search_channels(&self, pattern: &str) -> Vec<String> {
        let regex_set = RegexSet::new(self.channels.keys().map(|s| s.as_str())).unwrap();
        regex_set
            .matches(pattern)
            .into_iter()
            .map(|i| self.channels.keys().nth(i).unwrap().clone())
            .collect()
    }

    /// 🔹 Get the JSON Schema of a channel
    pub fn get_channel_schema(&self, name: &str) -> Option<String> {
        self.channels.get(name).map(|c| c.schema.clone())
    }

    /// 🔹 Add a message to a channel
    pub fn put_message(&mut self, name: &str, message: String) -> bool {
        if let Some(channel) = self.channels.get_mut(name) {
            channel.messages.push_back(message);
            true
        } else {
            false
        }
    }

    /// 🔹 Retrieve messages from a channel (FIFO order)
    pub fn get_messages(&self, name: &str, limit: usize) -> Vec<String> {
        if let Some(channel) = self.channels.get(name) {
            channel.messages.iter().take(limit).cloned().collect()
        } else {
            vec![]
        }
    }
}
