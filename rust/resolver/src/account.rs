//! Account related types that are used for deserializing JSON-RPC responses and notifications

use std::{ops::Deref, str::FromStr};

use base64::prelude::{Engine, BASE64_STANDARD};
use json::Deserialize;
use sdk::pubkey::Pubkey;
use serde::{de::Error as _, Deserializer};
use smallvec::SmallVec;

use crate::DELEGATION_PROGRAM_ID;

/// Wrapper around actual account state, used for deserialization
#[derive(Deserialize, Debug)]
#[serde(bound(deserialize = "'de: 'a"))]
pub struct AccountInfo<'a> {
    /// actual account state
    pub value: AccountValue<'a>,
}

impl<'a> Deref for AccountInfo<'a> {
    type Target = AccountValue<'a>;

    fn deref(&self) -> &Self::Target {
        &self.value
    }
}

/// State of account with extracted relevant fields.
#[derive(Deserialize, Debug)]
#[serde(bound(deserialize = "'de: 'a"))]
pub struct AccountValue<'a> {
    /// Account owner
    #[serde(deserialize_with = "deserialize_pubkey_from_base58")]
    pub owner: Pubkey,
    /// Current account balance in SOL
    pub lamports: u64,
    /// reference to underlying memory containing JSON serialization
    pub data: SmallVec<[&'a str; 2]>,
}

impl AccountValue<'_> {
    /// Check if owner account is Delegation Program, and that account is not closed
    pub fn is_delegated(&self) -> bool {
        self.owner == DELEGATION_PROGRAM_ID && self.lamports != 0
    }

    pub fn data(&self) -> Option<Vec<u8>> {
        let encoding = match self.data.len() {
            1 => "base58",
            2 => self.data.last().unwrap(),
            _ => {
                return None;
            }
        };

        let encoded = *self.data.first().unwrap();
        match encoding {
            "base58" => bs58::decode(encoded).into_vec().ok(),
            "base64" => BASE64_STANDARD.decode(encoded).ok(),
            "base64+zstd" => {
                let decoded = BASE64_STANDARD.decode(encoded).ok()?;
                zstd::decode_all(decoded.as_slice()).ok()
            }
            _ => None,
        }
    }
}

/// Deserialize solana Pubkey from base58 encoded string
pub fn deserialize_pubkey_from_base58<'de, D>(deserializer: D) -> Result<Pubkey, D::Error>
where
    D: Deserializer<'de>,
{
    let string = <&str as Deserialize>::deserialize(deserializer)?;
    Pubkey::from_str(string).map_err(D::Error::custom)
}

/// Find the PDA associated with the delegation record for given account
pub fn delegation_record_pda(pubkey: &Pubkey) -> Pubkey {
    let seeds: &[&[u8]] = &[b"delegation", pubkey.as_ref()];
    Pubkey::find_program_address(seeds, &DELEGATION_PROGRAM_ID).0
}
