use std::{collections::HashMap, net::SocketAddr, sync::Arc};

use easy_pv_datatypes::channel::ChannelState;
use tokio::sync::{Mutex, mpsc};

#[derive(Debug)]
pub enum SessionCommand {
    SendEcho(Vec<u8>),
    Shutdown,
    // Add others like CreateChannel, GetStats, etc.
}

pub type ConnectionMap = Arc<Mutex<HashMap<SocketAddr, mpsc::Sender<SessionCommand>>>>;

#[derive(Debug, Clone)]
pub struct ServerState {
    pub connections: ConnectionMap,
    pub channels: Arc<Mutex<HashMap<String, ChannelState>>>,
    pub logs: Arc<Mutex<Vec<String>>>,
}
