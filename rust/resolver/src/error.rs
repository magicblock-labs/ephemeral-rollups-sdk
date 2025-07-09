//! Errors used by router
use rpc_api::client_error;
use url::Url;

/// All errors that can be encountered during router operation
#[derive(thiserror::Error, Debug)]
pub enum Error {
    /// Route resolver error
    #[error("error resolving route")]
    Resolver(String),
    /// Error encountered during forwarding the request to upstream
    #[error("http error during request to remote: {0}")]
    HttpClient(#[from] reqwest::Error),
    /// Error making rpc request via solana client
    #[error("solana rpc-client error: {0}")]
    Rpc(#[from] Box<client_error::Error>),
    /// Error encountered during websocket connection handling
    #[error("websocket connection error: {0}")]
    Ws(#[from] websocket::Error),
    /// Internal router errors
    #[error("internal router error: {0}")]
    Internal(#[from] InternalError),
}

/// Internal router error
#[derive(thiserror::Error, Debug)]
pub enum InternalError {
    /// Provided url is invalid for the connection
    #[error("invalid connection url for {0}: {1}")]
    InvalidUrl(&'static str, Url),
    #[error("couldn't initialize the routing table, no routes available")]
    NoRoutesAvailable,
}
