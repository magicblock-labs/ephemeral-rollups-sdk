//! Configuration used by various modules of router

use std::time::Duration;

use json::Deserialize;
pub use sdk::commitment_config::CommitmentLevel;
use serde::{de::Error, Deserializer};
use url::Url;

/// General router configuration
#[derive(Deserialize)]
pub struct Configuration {
    /// configuration of client connections to base chain
    pub chain: Url,
    /// websocket connection pool configuration
    pub websocket: WebsocketConf,
    /// number of entries the delegations cache can hold
    /// this can be used to restrict memory usage by resolver
    /// so that it doesn't keep unecesary accounts around
    pub cache_size: usize,
    /// default commitment level to be used with rpc clients
    pub commitment: CommitmentLevel,
}

/// Configuration for the WebSocket connection.
#[derive(Deserialize, Clone)]
#[serde(rename_all = "kebab-case")]
pub struct WebsocketConf {
    /// The WebSocket endpoint URL.
    pub url: Url,
    /// The interval at which ping messages are sent to keep the connection alive.
    #[serde(deserialize_with = "deserialize_duration")]
    pub ping_interval: Duration,
}

/// Deserialize std::time::Duration from human readable string
pub fn deserialize_duration<'de, D>(deserializer: D) -> Result<Duration, D::Error>
where
    D: Deserializer<'de>,
{
    let string = String::deserialize(deserializer)?;
    humantime::parse_duration(&string).map_err(D::Error::custom)
}
