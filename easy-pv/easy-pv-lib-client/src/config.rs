use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct ClientConfig{
    pub udp_port: u16,
    pub tcp_port: u16,
}