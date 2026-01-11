#[cfg(feature = "anchor")]
use anchor_lang::prelude::*;

#[cfg(not(feature = "anchor"))]
use borsh::{BorshDeserialize, BorshSerialize};

use crate::solana_compat::solana::Pubkey;

#[cfg_attr(feature = "anchor", derive(AnchorSerialize, AnchorDeserialize))]
#[cfg_attr(not(feature = "anchor"), derive(BorshSerialize, BorshDeserialize))]
#[derive(Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Member {
    pub flags: u8,
    #[cfg_attr(
        feature = "serde",
        serde(with = "serde_with::As::<serde_with::DisplayFromStr>")
    )]
    pub pubkey: Pubkey,
}

// Flags for Member
pub const MEMBER_FLAG_DEFAULT: u8 = 0;
pub const MEMBER_FLAG_AUTHORITY: u8 = 1 << 0; // Member has authority privileges
pub const MEMBER_FLAG_TX_LOGS: u8 = 1 << 1; // Member can see transaction logs
pub const MEMBER_FLAG_TX_BALANCES: u8 = 1 << 2; // Member can see transaction balances

impl Member {
    pub fn is_authority(&self, user: &Pubkey) -> bool {
        self.flags & MEMBER_FLAG_AUTHORITY != 0 && &self.pubkey == user
    }

    pub fn can_see_tx_logs(&self, user: &Pubkey) -> bool {
        self.flags & MEMBER_FLAG_TX_LOGS != 0 && &self.pubkey == user
    }

    pub fn can_see_tx_balances(&self, user: &Pubkey) -> bool {
        self.flags & MEMBER_FLAG_TX_BALANCES != 0 && &self.pubkey == user
    }

    // Set multiple flags at once
    pub fn set_flags(&mut self, flags: u8) {
        self.flags |= flags;
    }
    // Remove multiple flags
    pub fn remove_flags(&mut self, flags: u8) {
        self.flags &= !flags;
    }
}
