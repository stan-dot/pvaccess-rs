use async_feature::Feature;
use features::{echo::EchoFeature, ping::PingFeature};
pub mod features;

pub mod async_feature;
pub mod config;
pub mod server;
pub mod state;

pub fn add(left: u64, right: u64) -> u64 {
    left + right
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }
}

pub fn register_features() -> Vec<Box<dyn Feature<Incoming = (), Outgoing = ()>>> {
    let mut features: Vec<Box<dyn Feature<Incoming = (), Outgoing = ()>>> = vec![];

    #[cfg(feature = "ping")]
    features.push(Box::new(PingFeature));

    #[cfg(feature = "echo")]
    features.push(Box::new(EchoFeature));

    features
}
