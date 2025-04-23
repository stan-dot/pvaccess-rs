use std::net::IpAddr;

use serde::Deserialize;

#[derive(Debug, Deserialize, Clone, Copy, Eq, PartialEq)]
pub struct ClientConfig{
    pub udp_host: IpAddr,
    pub udp_port: u16,
    pub tcp_port: u16,
    pub buffer_size: u32,
}