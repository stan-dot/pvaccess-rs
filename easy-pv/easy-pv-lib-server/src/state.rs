use std::{any::Any, collections::HashMap};

#[derive(Debug)]
pub struct ServerState {
    // clients: HashMap<ClientId, ClientState>,
    pub feature_state: HashMap<String, Box<dyn Any + Send + Sync>>,
}
