//! Websocket connection for handling cache maintenance subscriptions

use std::{
    collections::HashMap,
    sync::{atomic::Ordering, Arc},
};

use rpc::nonblocking::rpc_client::RpcClient;
use sdk::pubkey::Pubkey;
use tokio::sync::mpsc::UnboundedReceiver;

use crate::{
    config::WebsocketConf,
    http::update_account_state,
    websocket::message::{Notification, WebsocketMessage},
    DelegationStatus, DelegationsDB, DELEGATION_RECORD_SIZE,
};

use super::{base::WsConnectionBase, subscription::AccountSubscription};

const SLOT_SUBSCRIPTION: &str =
    r#"{ "jsonrpc": "2.0", "id": 4294967295, "method": "slotSubscribe" }"#;

/// Handle to a websocket connection
pub struct WsConnection {
    /// base websocket connection wrapper
    base: WsConnectionBase,
    /// Key-value store for delegated accounts
    db: DelegationsDB,
    /// subscription requests which were sent but not yet confirmed request ID -> subscription meta
    pending: HashMap<u64, AccountSubscription>,
    /// confirmed subscriptions, subscription ID -> subscription meta
    active: HashMap<u64, AccountSubscription>,
    /// unsubscription requests which were sent but not yet confirmed request ID -> subscription meta
    unsubs: HashMap<u64, AccountSubscription>,
    /// receiver of new subscriptions for newly encountered accounts
    rx: UnboundedReceiver<AccountSubscription>,
    /// HTTP client for base chain requests
    chain: Arc<RpcClient>,
}

impl WsConnection {
    /// Try to establish new websocket connection to endpoint
    pub async fn establish(
        config: WebsocketConf,
        chain: Arc<RpcClient>,
        rx: UnboundedReceiver<AccountSubscription>,
        db: DelegationsDB,
    ) -> crate::ResolverResult<Self> {
        let base = WsConnectionBase::new(config.url, config.ping_interval).await?;
        let pending = HashMap::new();
        let active = HashMap::new();
        let unsubs = HashMap::new();
        Ok(Self {
            base,
            db,
            pending,
            active,
            unsubs,
            rx,
            chain,
        })
    }

    /// Start handling websocket connection: processing notifications, and managing subscriptions
    pub async fn start(mut self) {
        // subcribe to slot, this creates some traffic on connection
        let _ = self.base.send(SLOT_SUBSCRIPTION).await;
        // convenience Result<T, E> unwrapper with reconnection on error
        macro_rules! check {
            ($result: expr) => {
                match $result {
                    Ok(value) => value,
                    Err(error) => {
                        tracing::warn!(%error, "websocket message handling");
                        self.reestablish().await;
                        continue;
                    }
                }
            };
        }

        loop {
            // use biased ordering to turn off select! RNG and handle events in prioritized manner
            tokio::select! {
                // process incoming websocket messages
                biased; msg = self.base.recv() => {
                    let payload = check!(msg);

                    // parse and handle received message
                    let msg = check!(WebsocketMessage::deserialize(&payload));
                    match msg {
                        WebsocketMessage::Subscribed(r) => {
                            if let Some(sub) = self.pending.remove(&r.id) {
                                tracing::info!(pubkey=%sub.pubkey, id=r.result, "subscribed to account");
                                sub.subscribed.store(true, Ordering::Release);
                                self.active.insert(r.result, sub);
                            }
                        }
                        WebsocketMessage::Unsubscribed(r) => {
                            if let Some(sub) = self.unsubs.remove(&r.id) {
                                sub.subscribed.store(false, Ordering::Release);
                                tracing::info!(pubkey=%sub.pubkey, "unsubscribed from account");
                            } else {
                                tracing::warn!(id=%r.id, "unsubscribed from unknown subscription");
                            }
                        }
                        WebsocketMessage::Notification(n) => {
                            match n {
                                Notification::Slot{ params } => {
                                    tracing::debug!(slot=params.result.slot, "slot received on ws");
                                }
                                Notification::Account{ params } => {
                                    let Some(account) = self.active.get(&params.subscription) else {
                                        tracing::warn!(sub=params.subscription,"received account update via unknown subscription");
                                        continue;
                                    };

                                    let mut should_unsubscribe = false;
                                    if !params.result.is_delegated() {
                                        // remove account from list of delegated ones
                                        if let Some(mut record) = self.db.get_async(&account.pubkey).await {
                                            record.get_mut().status = DelegationStatus::Undelegated;
                                        } else {
                                            should_unsubscribe = true;
                                        }
                                    } else {
                                        let Some(data) = params.result.data() else {
                                            tracing::warn!("account notification didn't contain data");
                                            continue
                                        };
                                        if data.len() != DELEGATION_RECORD_SIZE {
                                            tracing::warn!(size=data.len(), "wrong delegation record size");
                                            continue;
                                        }
                                        let mut buffer = [0; 32];
                                        buffer.copy_from_slice(&data[8..40]);
                                        let validator = Pubkey::new_from_array(buffer);
                                        if let Some(mut record) = self.db.get_async(&account.pubkey).await {
                                            record.get_mut().status = DelegationStatus::Delegated(validator);
                                        } else {
                                            should_unsubscribe = true;
                                        }
                                    }
                                    if should_unsubscribe {
                                        // infallible: checked above that subscription exists in self.active
                                        let account = self.active.remove(&params.subscription).unwrap();
                                        // once the account has been undelegated, its delegation
                                        // record is deleted and thus we are no longer interested
                                        // in it, and we can safely unsubscribe
                                        let msg = format!(
                                            r#"{{ "jsonrpc": "2.0", "id": {}, "method": "accountUnsubscribe", "params": [{}] }}"#
                                            , account.id, params.subscription);
                                        let _ = self.base.send(msg).await;
                                        self.unsubs.insert(account.id, account);
                                    }
                                }
                            }
                        }
                    }
                }
                // process subscription requests
                Some(sub) = self.rx.recv() => {
                    let msg = sub.ws();
                    self.pending.insert(sub.id, sub);
                    let _ = self.base.send(msg).await;
                }
                else => {
                    tracing::info!("ws connection is shutting down");
                    break;
                }
            }
        }
    }

    async fn reestablish(&mut self) {
        tracing::info!(
            subcount = self.active.len(),
            "reconnecting to websocket stream"
        );
        for sub in self.active.values_mut().chain(self.pending.values_mut()) {
            sub.subscribed.store(false, Ordering::Release);
        }
        'outer: loop {
            self.base.reconnect().await;
            // little hack to avoid extra allocations,
            // we are not leaving `reastablish` method
            // before connection is active and consistent
            // state is restored, so it's acceptable
            self.active.extend(self.pending.drain());
            let mut active = self.active.drain();
            while let Some((_, sub)) = active.next() {
                let msg = sub.ws();
                self.pending.insert(sub.id, sub);
                // realistically speaking, this should never happen
                if let Err(error) = self.base.send(msg).await {
                    tracing::warn!(%error, "error sending subscription on reconnect");
                    self.pending.extend(active.map(|(_, s)| (s.id, s)));
                    continue 'outer;
                }
            }
            // don't forget to resubscribe to slot updates
            if self.base.send(SLOT_SUBSCRIPTION).await.is_ok() {
                break;
            }
        }
        for sub in self.pending.values_mut() {
            let db = self.db.clone();
            let chain = self.chain.clone();
            let pubkey = sub.pubkey;
            // in order for reconnection to happen as fast as possible,
            // we spawn actual account fetching into separate tasks, that
            // way delegation status retrieval happens asynchronously
            tokio::spawn(update_account_state(chain, db, pubkey));
        }
        tracing::info!("reconnection to websocket stream succeeded");
    }
}
