use std::{any::Any, collections::HashMap};

pub struct ServerState {
    // clients: HashMap<ClientId, ClientState>,
    feature_state: HashMap<String, Box<dyn Any + Send + Sync>>,
}
