//! Websocket messages received from remote

use json::{Deserialize, JsonValueTrait};

use crate::account::AccountInfo;

/// Represents a WebSocket message received from the Solana blockchain.
/// can be either a result of subscription with ID or an actual state update
#[derive(Debug)]
pub enum WebsocketMessage<'a> {
    /// A subscription result message.
    Subscribed(SubscriptionResult),
    /// An unsubscription result message.
    Unsubscribed(UnsubscriptionResult),
    /// A notification message.
    Notification(Notification<'a>),
}

impl<'a> WebsocketMessage<'a> {
    /// Deserializes a WebSocket message from a byte buffer.
    pub fn deserialize(buffer: &'a [u8]) -> Result<Self, json::Error> {
        let result = json::lazyvalue::get(buffer, &["result"]);
        let msg = if let Ok(result) = result {
            if result.is_u64() {
                WebsocketMessage::Subscribed(json::from_slice::<SubscriptionResult>(buffer)?)
            } else {
                WebsocketMessage::Unsubscribed(json::from_slice::<UnsubscriptionResult>(buffer)?)
            }
        } else {
            WebsocketMessage::Notification(json::from_slice::<Notification>(buffer)?)
        };
        Ok(msg)
    }
}

/// Represents the parameters of a notification message.
#[derive(Deserialize, Debug)]
pub struct NotificationParams<T> {
    /// Result of notification
    pub result: T,
    /// Subscription ID
    pub subscription: u64,
}

/// Represents a notification message received from the Solana blockchain.
#[derive(Deserialize, Debug)]
#[serde(bound(deserialize = "'de: 'a"))]
#[serde(tag = "method")]
pub enum Notification<'a> {
    /// A slot notification.
    #[serde(rename = "slotNotification")]
    Slot {
        /// slot notification parameters
        params: NotificationParams<SlotInfo>,
    },
    /// An account notification.
    #[serde(rename = "accountNotification")]
    Account {
        /// account notification parameters
        params: NotificationParams<AccountInfo<'a>>,
    },
}

/// Represents a subscription result message.
#[derive(Deserialize, Debug)]
pub struct SubscriptionResult {
    /// ID of subscription request, not a subscription ID
    pub id: u64,
    /// resultant subscription ID
    pub result: u64,
}

/// Represents an unsubscription result message.
#[derive(Deserialize, Debug)]
pub struct UnsubscriptionResult {
    /// ID of unsubscription request
    pub id: u64,
}

/// Represents the payload of slot notification.
#[derive(Deserialize, Debug)]
pub struct SlotInfo {
    /// current slot
    pub slot: u64,
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use super::*;
    use json::from_slice;
    use sdk::Pubkey;

    const ACCOUNT_NOTIFICATION: &[u8] = br#"{
            "jsonrpc": "2.0",
            "method": "accountNotification",
            "params": {
                "result": {
                    "context": {
                        "slot": 5199307
                    },
                    "value": {
                        "data": [
                            "11116bv5nS2h3y12kD1yUKeMZvGcKLSjQgX6BeV7u1FrjeJcKfsHPXHRDEHrBesJhZyqnnq9qJeUuF7WHxiuLuL5twc38w2TXNLxnDbjmuR",
                            "base58"
                        ],
                        "executable": false,
                        "lamports": 33594,
                        "owner": "11111111111111111111111111111111",
                        "rentEpoch": 635,
                        "space": 80
                    }
                },
                "subscription": 23784
            }
        }"#;
    const SUBSCRIPTION_RESULT: &[u8] = br#"{ "jsonrpc": "2.0", "result": 23784, "id": 1 }"#;
    const UNSUBSCRIPTION_RESULT: &[u8] = br#"{ "jsonrpc": "2.0", "result": true, "id": 1 }"#;
    const SLOT_NOTIFICATION: &[u8] = br#"{
        "jsonrpc": "2.0",
        "method": "slotNotification",
        "params": {
            "result": {
                "parent": 75,
                "root": 44,
                "slot": 76
            },
            "subscription": 0
        }
    }"#;

    #[test]
    fn test_deserialize_subscription_result() {
        let message: SubscriptionResult = from_slice(SUBSCRIPTION_RESULT).unwrap();
        assert_eq!(message.id, 1);
        assert_eq!(message.result, 23784);
    }

    #[test]
    fn test_deserialize_unsubscription_result() {
        let message: UnsubscriptionResult = from_slice(UNSUBSCRIPTION_RESULT).unwrap();
        assert_eq!(message.id, 1);
    }

    #[test]
    fn test_deserialize_account_notification() {
        let message: Notification = from_slice(ACCOUNT_NOTIFICATION).unwrap();
        if let Notification::Account { params } = message {
            assert_eq!(params.subscription, 23784);
            let AccountInfo { value } = params.result;
            assert_eq!(value.lamports, 33594);
            assert_eq!(
                value.owner,
                Pubkey::from_str("11111111111111111111111111111111").unwrap()
            );
        } else {
            panic!("Invalid message type");
        }
    }

    #[test]
    fn test_deserialize_slot_notification() {
        let message: Notification = from_slice(SLOT_NOTIFICATION).unwrap();
        if let Notification::Slot { params } = message {
            assert_eq!(params.subscription, 0);
            let SlotInfo { slot } = params.result;
            assert_eq!(slot, 76);
        } else {
            panic!("Invalid message type");
        }
    }

    #[test]
    fn test_deserialize_ws_message() {
        let mut msg = WebsocketMessage::deserialize(SUBSCRIPTION_RESULT);
        assert!(matches!(msg, Ok(WebsocketMessage::Subscribed(_))));
        msg = WebsocketMessage::deserialize(UNSUBSCRIPTION_RESULT);
        assert!(matches!(msg, Ok(WebsocketMessage::Unsubscribed(_))));
        msg = WebsocketMessage::deserialize(ACCOUNT_NOTIFICATION);
        assert!(matches!(
            msg,
            Ok(WebsocketMessage::Notification(Notification::Account { .. }))
        ));
        msg = WebsocketMessage::deserialize(SLOT_NOTIFICATION);
        assert!(matches!(
            msg,
            Ok(WebsocketMessage::Notification(Notification::Slot { .. }))
        ));
    }
}
