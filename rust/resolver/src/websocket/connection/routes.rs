//! Websocket connection for handling cache maintenance subscriptions

use std::{sync::Arc, time::Duration};

use borsh::BorshDeserialize;
use mdp::state::record::ErRecord;
use rpc::nonblocking::rpc_client::RpcClient;

use crate::{
    account::ProgramAccountValue,
    config::WebsocketConf,
    http::fetch_domain_records,
    websocket::{
        base::WsConnectionBase,
        message::{Notification, WebsocketMessage},
    },
    RoutingTable,
};

/// Handle to a websocket connection
pub struct WsRoutesConnection {
    /// base websocket connection wrapper
    base: WsConnectionBase,
    /// Key-value store for delegated accounts
    routes: RoutingTable,
    /// HTTP client for base chain requests
    chain: Arc<RpcClient>,
}

impl WsRoutesConnection {
    /// Try to establish new websocket connection to endpoint
    pub async fn establish(
        config: WebsocketConf,
        chain: Arc<RpcClient>,
        routes: RoutingTable,
    ) -> crate::ResolverResult<Self> {
        let base = WsConnectionBase::new(config.url, config.ping_interval).await?;
        Ok(Self {
            base,
            routes,
            chain,
        })
    }

    /// Start handling websocket connection: processing ER record update notifications
    pub async fn start(mut self) {
        // subcribe to accounts of magic domain program
        let _ = self.base.send(Self::generate_subscription()).await;
        loop {
            match self.base.recv().await {
                Ok(payload) => {
                    let Ok(msg) = WebsocketMessage::deserialize(&payload) else {
                        tracing::warn!(
                            "received unknown websocket message format on routes connection: {}",
                            String::from_utf8_lossy(&payload)
                        );
                        continue;
                    };
                    match msg {
                        WebsocketMessage::Subscribed(sub) => {
                            tracing::info!(
                                "subcribed to MDP program notifications, sub id: {}",
                                sub.result
                            );
                        }
                        WebsocketMessage::Notification(Notification::Program { params }) => {
                            let ProgramAccountValue { pubkey, account } = params.result.value;
                            tracing::debug!("received ER record updated for {pubkey}");
                            let Some(data) = account.data() else {
                                continue;
                            };
                            let record = match ErRecord::try_from_slice(&data) {
                                Ok(record) => record,
                                Err(err) => {
                                    tracing::warn!(
                                        "failed to deserialize ER record from account data: {err}"
                                    );
                                    continue;
                                }
                            };
                            Self::update_routing_table(&self.routes, record);
                        }
                        unexpected => {
                            tracing::warn!(
                                "received unexpected websocket message on routes connection: {unexpected:?}"
                            );
                        }
                    }
                }
                Err(error) => {
                    tracing::warn!("routes websocket has failed: {error}");
                    self.reestablish().await;
                    continue;
                }
            };
        }
    }

    async fn reestablish(&mut self) {
        loop {
            self.base.reconnect().await;
            if self.base.send(Self::generate_subscription()).await.is_ok() {
                break;
            }
            tokio::time::sleep(Duration::from_secs(1)).await;
        }
        let routes = self.routes.clone();
        let client = self.chain.clone();
        tokio::spawn(async move {
            let mut attempts = 0;
            let records = loop {
                match fetch_domain_records(&client).await {
                    Ok(records) => break records,
                    Err(err) => {
                        attempts += 1;
                        tracing::warn!(
                            "failed to refetch domain registry records: {err}, attemt: {attempts}"
                        );
                        if attempts > 4 {
                            return;
                        }
                        tokio::time::sleep(Duration::from_secs(2 * attempts)).await;
                        continue;
                    }
                }
            };
            for r in records {
                Self::update_routing_table(&routes, r);
            }
        });
        tracing::info!("reconnection to the routes websocket stream succeeded");
    }

    fn update_routing_table(routes: &RoutingTable, record: ErRecord) {
        let identity = record.identity();
        let address_is_the_same = routes
            .read()
            .get(identity)
            .map(|client| client.url() == record.addr())
            .unwrap_or_default();
        if address_is_the_same {
            return;
        }

        let client = Arc::new(RpcClient::new(record.addr().to_owned()));
        routes.write().insert(*identity, client);
    }

    fn generate_subscription() -> String {
        format!(
            r#"
            {{
                "jsonrpc": "2.0",
                "id": 1,
                "method": "getProgramAccounts",
                "params": ["{}"]
            }}
            "#,
            mdp::id()
        )
    }
}
