use crate::compat::{self, Pubkey};

#[cfg(feature = "anchor-support")]
#[allow(unused_imports)]
use crate::compat::anchor_lang;
#[cfg(feature = "anchor-support")]
use crate::compat::anchor_lang::{AnchorDeserialize, AnchorSerialize};
#[cfg(feature = "anchor-support")]
#[allow(unused_imports)]
use crate::compat::borsh;
#[cfg(not(feature = "anchor-support"))]
use crate::compat::borsh::{BorshDeserialize, BorshSerialize};

// IMPORTANT: Keep Pubkey unqualified in Anchor IDL-derived structs. Anchor's
// idl-build recognizes bare Pubkey as the native IDL pubkey type, while
// compat::Pubkey is treated as a custom type that must implement IdlBuild.
#[cfg_attr(feature = "anchor-support", derive(AnchorSerialize, AnchorDeserialize))]
#[cfg_attr(
    not(feature = "anchor-support"),
    derive(BorshSerialize, BorshDeserialize)
)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Member {
    pub flags: u8,
    pub pubkey: Pubkey,
}

#[cfg_attr(feature = "anchor-support", derive(AnchorSerialize, AnchorDeserialize))]
#[cfg_attr(
    not(feature = "anchor-support"),
    derive(BorshSerialize, BorshDeserialize)
)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MembersArgs {
    pub members: Option<Vec<Member>>,
}

// Flags for Member
pub const AUTHORITY_FLAG: u8 = 1 << 0; // Member has authority privileges
pub const TX_LOGS_FLAG: u8 = 1 << 1; // Member can see transaction logs
pub const TX_BALANCES_FLAG: u8 = 1 << 2; // Member can see transaction balances
pub const TX_MESSAGE_FLAG: u8 = 1 << 3; // Member can see transaction messages
pub const ACCOUNT_SIGNATURES_FLAG: u8 = 1 << 4; // Member can see account signatures

impl Member {
    pub fn is_authority(&self, user: &compat::Pubkey) -> bool {
        self.flags & AUTHORITY_FLAG != 0 && &self.pubkey == user
    }

    pub fn can_see_tx_logs(&self, user: &compat::Pubkey) -> bool {
        self.flags & TX_LOGS_FLAG != 0 && &self.pubkey == user
    }

    pub fn can_see_tx_balances(&self, user: &compat::Pubkey) -> bool {
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
