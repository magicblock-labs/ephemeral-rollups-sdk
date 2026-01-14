#[cfg(feature = "anchor")]
use anchor_lang::prelude::*;

#[cfg(not(feature = "anchor"))]
use borsh::{BorshDeserialize, BorshSerialize};

use crate::solana_compat::solana::Pubkey;

#[cfg_attr(feature = "anchor", derive(AnchorSerialize, AnchorDeserialize))]
#[cfg_attr(not(feature = "anchor"), derive(BorshSerialize, BorshDeserialize))]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Member {
    pub flags: u8,
    pub pubkey: Pubkey,
}

#[cfg_attr(feature = "anchor", derive(AnchorSerialize, AnchorDeserialize))]
#[cfg_attr(not(feature = "anchor"), derive(BorshSerialize, BorshDeserialize))]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MembersArgs {
    pub members: Option<Vec<Member>>,
}

// Flags for Member
pub const AUTHORITY_FLAG: u8 = 1 << 0; // Member has authority privileges
pub const TX_LOGS_FLAG: u8 = 1 << 1; // Member can see transaction logs
pub const TX_BALANCES_FLAG: u8 = 1 << 2; // Member can see transaction balances

impl Member {
    pub fn is_authority(&self, user: &Pubkey) -> bool {
        self.flags & AUTHORITY_FLAG != 0 && &self.pubkey == user
    }

    pub fn can_see_tx_logs(&self, user: &Pubkey) -> bool {
        self.flags & TX_LOGS_FLAG != 0 && &self.pubkey == user
    }

    pub fn can_see_tx_balances(&self, user: &Pubkey) -> bool {
        self.flags & TX_BALANCES_FLAG != 0 && &self.pubkey == user
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
