//! A utility SDK to facilitate route resolution for a subset of solana JSON-RPC requests

use std::{
    collections::HashMap,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
};

use parking_lot::RwLock;

use config::Configuration;
use error::Error;
use http::{fetch_account_state, fetch_domain_records, update_account_state};
use rpc::nonblocking::rpc_client::RpcClient;
use scc::{hash_cache::Entry, HashCache};
use sdk::{commitment_config::CommitmentConfig, pubkey::Pubkey, transaction::Transaction};
use tokio::sync::mpsc::{unbounded_channel, UnboundedSender};
use websocket::{
    connection::{delegations::WsDelegationsConnection, routes::WsRoutesConnection},
    subscription::AccountSubscription,
};

/// Mapping between validator(ER) identity and solana rpc client, which is
/// configured with the URL, via which the this particular ER can be reached
/// NOTE: we use RwLock with std::HashMap instead of concurrent HashMap, as this table is not
/// supposed to be modified too often, so RwLock is faster for mostly read workload
type RoutingTable = Arc<RwLock<HashMap<Pubkey, Arc<RpcClient>>>>;
/// Limited capacity (LRU) cache, mapping between an account's
/// pubkey and it's current delegation status as observed by resolver
type DelegationsDB = Arc<HashCache<Pubkey, DelegationRecord>>;
/// Conveniece wrapper for results with possible resolver errors
type ResolverResult<T> = Result<T, Error>;

const DELEGATION_PROGRAM_ID: Pubkey = ephemeral_rollups_sdk::id();
/// The fixed size of delegation record account's data,
/// NOTE: this value should be updated if the ABI of delegation
/// program changes in the future, that will affect the size
const DELEGATION_RECORD_SIZE: usize = 88;

/// Connection resolver, the type is cheaply clonable and thus a single instance should be
/// initialized and cloned between threads if necessary
#[derive(Clone)]
pub struct Resolver {
    routes: RoutingTable,
    delegations: DelegationsDB,
    chain: Arc<RpcClient>,
    delegations_tx: UnboundedSender<AccountSubscription>,
}

/// Delegation status of account
#[derive(Clone, Copy)]
pub enum DelegationStatus {
    /// Account is delegated to validator indicated by pubkey
    Delegated(Pubkey),
    /// Account is available for modification on chain
    Undelegated,
}

/// Wrapper around delegation status, with additional flag to keep track of subscription state
struct DelegationRecord {
    /// current delegation status of account, last observed by resolver
    status: DelegationStatus,
    /// indicator, whether active websocket subscription exists for account updates, to track its
    /// delegation status
    subscribed: Arc<AtomicBool>,
}

impl Resolver {
    /// Initialize the resolver with the provided configuration
    pub async fn new(config: Configuration) -> ResolverResult<Self> {
        Self::new_custom(config, true, None).await
    }
    /// Initialize the resolver:
    /// 1. fetch routes from chain (if use_on_chain_routes is true)
    /// 2. add custom routes to routing table, if any
    /// 3. subscribe to on-chain route updates (if use_on_chain_routes is true)
    /// 4. creating websocket connection to base chain for delegation status tracking of accounts
    pub async fn new_custom(
        config: Configuration,
        use_on_chain_routes: bool,
        custom_routes: Option<HashMap<Pubkey, String>>,
    ) -> ResolverResult<Self> {
        let commitment = CommitmentConfig {
            commitment: config.commitment,
        };
        let chain = Arc::new(RpcClient::new(config.chain.to_string()));

        let mut routes: HashMap<_, _> = if use_on_chain_routes {
            fetch_domain_records(&chain)
                .await?
                .into_iter()
                .map(|record| {
                    (
                        *record.identity(),
                        RpcClient::new_with_commitment(record.addr().to_string(), commitment)
                            .into(),
                    )
                })
                .collect()
        } else {
            Default::default()
        };

        routes.extend(
            custom_routes
                .into_iter()
                .flatten()
                .map(|(k, v)| (k, RpcClient::new_with_commitment(v, commitment).into())),
        );

        let routes = Arc::new(RwLock::new(routes));

        let delegations = Arc::new(HashCache::with_capacity(128, config.cache_size.max(256)));
        let (delegations_tx, rx) = unbounded_channel();
        let delegations_ws = WsDelegationsConnection::establish(
            config.websocket.clone(),
            chain.clone(),
            rx,
            delegations.clone(),
        )
        .await?;

        tokio::spawn(delegations_ws.start());

        if use_on_chain_routes {
            let routes_ws =
                WsRoutesConnection::establish(config.websocket, chain.clone(), routes.clone())
                    .await?;
            tokio::spawn(routes_ws.start());
        }

        Ok(Self {
            chain,
            delegations,
            delegations_tx,
            routes,
        })
    }

    /// Start tracking account's delegation status, this is achieved by fetching the delegation
    /// record for the account (if it exists) and subscribing to updates of its state. The existence
    /// of the delegation record is a proof that account has been delegated, and it contains critical
    /// information like the identity of validator, to which the account was delegated
    pub async fn track_account(&self, pubkey: Pubkey) -> ResolverResult<DelegationStatus> {
        let chain = self.chain.clone();
        match self.delegations.entry(pubkey) {
            Entry::Vacant(e) => {
                let subscribed = Arc::new(AtomicBool::default());
                let record = DelegationRecord {
                    status: DelegationStatus::Undelegated,
                    subscribed: subscribed.clone(),
                };
                e.put_entry(record);
                let db = self.delegations.clone();
                let subscription = AccountSubscription::new(pubkey, subscribed);
                let status = update_account_state(chain, db, pubkey).await?;
                let _ = self.delegations_tx.send(subscription);
                Ok(status)
            }
            Entry::Occupied(e) => {
                // return cached status, only if subscription exists
                if e.subscribed.load(Ordering::Acquire) {
                    Ok(e.status)
                } else {
                    // otherwise refetch fresh version from chain, to avoid stale cache issue
                    fetch_account_state(chain, pubkey).await
                }
            }
        }
    }

    /// Resolve connection for given account, if account has been delegated (as observed by
    /// resolver), then the returned client is configured to connect to corresponding ER
    /// instance, otherwise the client will connect to base layer chain
    pub async fn resolve(&self, pubkey: &Pubkey) -> ResolverResult<Arc<RpcClient>> {
        let status = self.resolve_status(pubkey).await?;
        self.resolve_client(status)
    }

    /// Resolve connection for given transaction, if any of the accounts have been delegated
    /// (as observed by resolver), then the resolver will check that all the writable accounts in
    /// transaction have been delegated to the same ER, if validation is successful, the returned
    /// client is configured to connect to this common ER. If none of the accounts are delegated then
    /// the returned client is configured to connect to base layer chain. If conflict in delegation
    /// is found, i.e. writable accounts are delegated to different ERs, then error is returned as
    /// connection resolution is impossible for such a case.
    pub async fn resolve_for_transaction(
        &self,
        tx: &Transaction,
    ) -> ResolverResult<Arc<RpcClient>> {
        let mut statuses = Vec::new();
        for (i, acc) in tx.message.account_keys.iter().enumerate() {
            if tx.message.is_maybe_writable(i, None) {
                statuses.push(self.resolve_status(acc).await?);
            }
        }
        let mut validator = None;
        for s in statuses {
            let DelegationStatus::Delegated(v1) = s else {
                continue;
            };
            let Some(v2) = validator.replace(v1) else {
                continue;
            };
            if v1 != v2 {
                return Err(Error::Resolver(format!(
                    "transaction accounts delegated to different validators: {v1} <> {v2}"
                )));
            }
        }
        if let Some(v) = validator.map(DelegationStatus::Delegated) {
            return self.resolve_client(v);
        }
        Ok(self.chain.clone())
    }

    /// Get current delegation status for account, either from cache or
    /// from chain (if account is encoutered for the first time)
    async fn resolve_status(&self, pubkey: &Pubkey) -> ResolverResult<DelegationStatus> {
        if let Some(record) = self.delegations.get(pubkey) {
            if record.get().subscribed.load(Ordering::Acquire) {
                // only return cached status if websocket subscription exists
                Ok(record.get().status)
            } else {
                // fetch from chain otherwise
                fetch_account_state(self.chain.clone(), *pubkey).await
            }
        } else {
            self.track_account(*pubkey).await
        }
    }

    /// Depending on delegation status, return appropriate RpcClient,
    /// which can be used to perform requests for account involved
    fn resolve_client(&self, status: DelegationStatus) -> ResolverResult<Arc<RpcClient>> {
        match status {
            DelegationStatus::Delegated(validator) => {
                let guard = self.routes.read();
                let client = guard.get(&validator).ok_or(Error::Resolver(format!(
                    "url not found for validator: {validator}"
                )))?;
                Ok(client.clone())
            }
            DelegationStatus::Undelegated => Ok(self.chain.clone()),
        }
    }
}

mod account;
pub mod config;
mod error;
mod http;
mod websocket;
