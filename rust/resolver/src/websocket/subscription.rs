//! Types for working with WebSocket subscription messages in the Solana blockchain.

use std::sync::{
    atomic::{AtomicBool, AtomicU64, Ordering},
    Arc,
};

use sdk::pubkey::Pubkey;

use crate::account::delegation_record_pda;

/// Represents a websocket subscription to an account on the Solana blockchain.
pub struct AccountSubscription {
    /// JSON-RPC ID of request sent to upstream, used for both HTTP and WS
    pub id: u64,
    /// Indicator of presence of an active websocket subscription
    pub subscribed: Arc<AtomicBool>,
    /// Solana pubkey of account
    pub pubkey: Pubkey,
}

impl AccountSubscription {
    /// Creates a new `AccountSubscription` for the given `pubkey`.
    pub fn new(pubkey: Pubkey, subscribed: Arc<AtomicBool>) -> Self {
        /// Generates a unique request ID.
        fn id() -> u64 {
            static COUNTER: AtomicU64 = AtomicU64::new(0);
            COUNTER.fetch_add(1, Ordering::Relaxed)
        }
        Self {
            id: id(),
            pubkey,
            subscribed,
        }
    }

    /// Generate JSON-RPC request for websocket subscription
    pub fn ws(&self) -> Vec<u8> {
        self.json("accountSubscribe")
    }

    /// Returns a JSON representation (as slice) of the account request
    fn json(&self, method: &str) -> Vec<u8> {
        let value = json::json!({
            "jsonrpc": "2.0",
            "id": self.id,
            "method": method,
            "params": [
                // we don't use the account itself as a subscription target, but rather its
                // delegation record PDA, which allows us to obtain some extra data, like
                // identity of the validator which was used in the delegation process, and still
                // uniquely identify delegated accoounts
                delegation_record_pda(&self.pubkey).to_string(),
                {
                    "commitment": "confirmed",
                    // use the most compact form to reduce latency on network transmissions
                    "encoding": "base64+zstd"
                }
            ]
        });
        json::to_vec(&value).expect("acc sub should always serialize")
    }
}
