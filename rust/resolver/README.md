## Connection Resolver

Quickstart and integration guide:
https://docs.magicblock.gg/pages/get-started/how-integrate-your-program/quickstart

The Connection Resolver is a specialized library designed to facilitate the
resolution of RPC connections for Solana blockchain requests. It dynamically
determines the appropriate RPC client for processing requests based on the
delegation status of the accounts involved. This is accomplished by maintaining
an up-to-date record of account delegation statuses through real-time
synchronization with the Solana base chain, achieved via WebSocket
subscriptions or on-demand data retrieval.

Upon encountering a new account through the `resolve*` or `track_account`
functions, the Resolver fetches the account's delegation status directly from
the blockchain and initiates a WebSocket subscription to capture subsequent
updates. This ensures that the Resolver reflects the most current state of the
blockchain (which acts as a single source of truth), enabling it to deliver the
appropriate RPC client for any request involving a given account. This
mechanism allows developers to seamlessly direct transactions and requests to
the correct endpoints, thereby facilitating the interaction with the ephemeral
rollups.


### Basic Setup

To begin using the Connection Resolver, configure it with the necessary parameters including the base chain, websocket, and optionally custom routing table for ER nodes:

```rust
use magic_resolver::config::{Configuration, WebsocketConf};
use magic_resolver::Resolver;
use std::{collections::HashMap, time::Duration};
use solana_sdk::pubkey::Pubkey;

const DEVNET: &str = "https://api.devnet.solana.com/";

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let config = Configuration {
        chain: DEVNET.parse()?,
        websocket: WebsocketConf {
            url: "wss://api.devnet.solana.com".parse()?,
            ping_interval: Duration::from_secs(3),
        },
        cache_size: 1024,
    };

    let routes = {
        let mut table = HashMap::new();
        table.insert(
            Pubkey::from_str("11111111111111111111111111111111").unwrap(),
            "https://devnet.magicblock.app/".into(),
        );
        // Add additional validators as needed
        table
    };

    // this will use the union of on chain and custom routes
    let resolver = Resolver::new_custom(config, true, Some(routes)).await?;
    // additionally if you only want to use custom routes
    // let resolver = Resolver::new_custom(config, false, Some(routes)).await?;
    // or if you only want to use on chain routes
    // let resolver = Resolver::new(config).await?;

    Ok(())
}
```

### Tracking Account Delegation

To track an account's delegation status, use the `track_account` method. This will cache the delegation status and set up a WebSocket subscription for updates:

```rust
use solana_sdk::pubkey::Pubkey;

let pda = Pubkey::from_str("5RgeA5P8bRaynJovch3zQURfJxXL3QK2JYg1YamSvyLb").unwrap();
resolver.track_account(pda).await?;
```
Note that, the utilization of this method is optional and only serves to decrease the latency of `resolve*` methods when they first encouter any given account.

### Resolving Connection for a Single Account

You can resolve the appropriate RPC client for a specific account using its public key:

```rust
let client = resolver.resolve(&pda).await?;
println!("Resolved client URL: {}", client.url());
```

### Resolving Connection for a Transaction

The resolver can also determine the correct RPC endpoint for a transaction, ensuring all writable accounts are delegated consistently:

```rust
use solana_sdk::transaction::Transaction;
use solana_sdk::instruction::{AccountMeta, Instruction};

let increment_instruction = Instruction::new_with_bincode(
    Pubkey::from_str("852a53jomx7dGmkpbFPGXNJymRxywo3WsH1vusNASJRr").unwrap(),
    &[], // No instruction data
    vec![AccountMeta::new(pda, false)],
);

// no need to sign, as we don't have blockhash yet
let tx = Transaction::new_with_payer(
    &[increment_instruction],
    Some(&payer.pubkey()),
);

// the client can be then used to fetch latest blockhash, 
// sign and send the transaction to appropriate endpoint
let client = resolver.resolve_for_transaction(&tx).await?;
println!("Resolved client URL for transaction: {}", client.url());
```

