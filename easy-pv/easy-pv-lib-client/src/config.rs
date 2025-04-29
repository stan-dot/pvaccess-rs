use std::{
    fmt::{self},
    net::IpAddr,
};

use serde::Deserialize;

#[derive(Debug, Deserialize, Clone, Copy, Eq, PartialEq)]
pub struct ClientConfig {
    pub udp_host: IpAddr,
    pub udp_port: u16,
    pub tcp_port: u16,
    pub buffer_size: u32,
    pub introspection_registry_max_size: i16
}

impl fmt::Display for ClientConfig {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.buffer_size)
    }
}
