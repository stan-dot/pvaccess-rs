use config::{Config as RawConfig, File};
use serde::Deserialize;
use std::env;
use std::net::{IpAddr, SocketAddr};
use tracing::{info};

#[derive(Debug, Deserialize, Clone, Copy)]
pub struct AppConfig {
    pub beacon: BeaconConfig,
    pub websocket: ServerConfig,
    pub network: ServerConfig,
    pub connection_validation: ConnectionValidationParams,
}

#[derive(Debug, Deserialize, Copy, Clone)]
pub struct ConnectionValidationParams {
    pub receive_buffer_size: u32,
    pub introspection_registry_max_size: u16,
}

#[derive(Debug, Deserialize, Copy, Clone)]
pub struct BeaconConfig {
    pub udp_server_config: ServerConfig,
    pub udp_initial_interval: u64,
    pub udp_long_term_interval: u64,
}

#[derive(Debug, Deserialize, Clone, Copy)]
pub struct ServerConfig {
    pub host: IpAddr,
    pub port: u16,
}

impl ServerConfig {
    pub fn socket_addr(&self) -> SocketAddr {
        SocketAddr::new(self.host, self.port)
    }
}

impl AppConfig {
    pub fn new() -> Self {
        let config_path = env::var("CONFIG_PATH").unwrap_or_else(|_| {
            const DEV_PATH: &str = "easy-pv-server/dev-server";
            DEV_PATH.to_string()
        });

        info!("Loading config from: {}", config_path);

        let settings = RawConfig::builder()
            .add_source(File::with_name(&config_path))
            .build()
            .expect("Failed to load pv-server configuration");

        settings
            .try_deserialize()
            .expect("Failed to parse configuration")
    }
}
