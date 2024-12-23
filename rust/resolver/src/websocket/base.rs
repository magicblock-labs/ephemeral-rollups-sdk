use std::time::Duration;

use futures::{SinkExt, StreamExt};
use tokio::{net::TcpStream, time::Interval};
use url::Url;
use websocket::{ClientBuilder, MaybeTlsStream, Message, Payload, WebSocketStream};

use crate::{error::InternalError, ResolverResult};

pub type WsStream = WebSocketStream<MaybeTlsStream<TcpStream>>;

/// Base websocket connection, used for abstracting lower level details like ws pings
pub struct WsConnectionBase {
    /// actual websocket connection stream over TCP
    inner: WsStream,
    /// new connection builder, used for reconnection events
    builder: ClientBuilder<'static>,
    /// periodicity with which to send PING frames to websocket server
    /// acts as connection health checker
    ping: Interval,
    /// endpoint to which the connection is established
    url: Url,
}

impl WsConnectionBase {
    pub async fn new(url: Url, ping: Duration) -> ResolverResult<Self> {
        let builder = ClientBuilder::new()
            .uri(url.as_str())
            .map_err(|_| InternalError::InvalidUrl("websocket", url.clone()))?;
        let (inner, _) = builder.connect().await?;
        let ping = tokio::time::interval(ping);
        Ok(Self {
            inner,
            builder,
            ping,
            url,
        })
    }

    pub async fn recv(&mut self) -> Result<Payload, websocket::Error> {
        loop {
            tokio::select! {
                Some(msg) = self.inner.next() => {
                    let msg = msg.inspect_err(|error| tracing::warn!(%error, "failed to receive on websocket"))?;
                    if msg.is_ping() || msg.is_pong() {
                        continue;
                    }
                    if msg.is_close() {
                        tracing::warn!("remote host close ws connection");
                        break Err(websocket::Error::AlreadyClosed);
                    }
                    break Ok(msg.into_payload())
                }
                _ = self.ping.tick() => {
                    let msg = Message::ping("ping");
                    self.inner.send(msg).await?;
                }
                else => {
                    break Err(websocket::Error::AlreadyClosed);
                }
            }
        }
    }

    pub async fn send<P: Into<Payload>>(&mut self, payload: P) -> Result<(), websocket::Error> {
        let msg = Message::text(payload);
        self.inner
            .send(msg)
            .await
            .inspect_err(|error| tracing::warn!(%error, "failed to send websocket message"))
    }

    pub async fn reconnect(&mut self) {
        let attempt = 1;
        let inner = loop {
            match self.builder.connect().await {
                Ok((ws, _)) => break ws,
                Err(error) => {
                    tracing::warn!(
                        attempt,
                        %error,
                        url=%self.url.as_str(),
                        "failed to reconnect to websocket"
                    );
                }
            }
            tokio::time::sleep(Duration::from_secs(3)).await;
        };
        self.inner = inner;
    }
}
