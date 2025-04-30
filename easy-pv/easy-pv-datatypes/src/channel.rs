use std::collections::VecDeque;
use std::net::SocketAddr;
use std::time::{Instant, SystemTime};

use crate::pv_fielddesc::FieldDesc;

#[derive(Debug)]
pub struct ChannelState {
    pub name: String,
    pub schema: FieldDesc,                         // Channel schema
    pub messages: VecDeque<(SystemTime, Vec<u8>)>, // Timestamped raw payloads
    pub subscribers: Vec<SocketAddr>,              // Connected clients
    pub last_updated: Instant,                     // For monitoring freshness
    pub capacity: usize,                           // Max buffer size
}

impl ChannelState {
    pub fn new(name: String, schema: FieldDesc, capacity: usize) -> Self {
        Self {
            name,
            schema,
            messages: VecDeque::with_capacity(capacity),
            subscribers: Vec::new(),
            last_updated: Instant::now(),
            capacity,
        }
    }

    pub fn push_message(&mut self, payload: Vec<u8>) {
        if self.messages.len() == self.capacity {
            self.messages.pop_front();
        }
        self.messages.push_back((SystemTime::now(), payload));
        self.last_updated = Instant::now();
    }

    pub fn add_subscriber(&mut self, client: SocketAddr) {
        if !self.subscribers.contains(&client) {
            self.subscribers.push(client);
        }
    }

    pub fn remove_subscriber(&mut self, client: SocketAddr) {
        self.subscribers.retain(|c| c != &client);
    }
}
