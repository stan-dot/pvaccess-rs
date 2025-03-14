use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub enum MsgType {
    Echo,
    ConnectionValidation,
    Chat,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Msg {
    pub msg_type: MsgType,
    pub content: String,
}
