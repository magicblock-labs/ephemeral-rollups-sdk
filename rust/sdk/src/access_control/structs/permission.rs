use crate::access_control::structs::Member;
use crate::compat::{self, Pubkey};
use crate::consts::PERMISSION_PROGRAM_ID;

// IMPORTANT: Keep Pubkey unqualified in Anchor IDL-derived structs. Anchor's
// idl-build recognizes bare Pubkey as the native IDL pubkey type, while
// compat::Pubkey is treated as a custom type that must implement IdlBuild.
#[cfg(feature = "anchor")]
use anchor_lang::prelude::*;

#[cfg(not(feature = "anchor"))]
use crate::compat::borsh::{self, BorshDeserialize, BorshSerialize};

pub const PERMISSION_SEED: &[u8] = b"permission:";

#[cfg_attr(feature = "anchor", derive(AnchorSerialize, AnchorDeserialize))]
#[cfg_attr(not(feature = "anchor"), derive(BorshSerialize, BorshDeserialize))]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Permission {
    pub discriminator: u8,
    pub bump: u8,
    pub permissioned_account: Pubkey,
    pub members: Option<Vec<Member>>,
}

impl Permission {
    /// Prefix values used to generate a PDA for this account.
    ///
    /// Values are positional and appear in the following order:
    ///
    ///   0. `PERMISSION_SEED`
    ///   1. permissioned_account (`compat::Pubkey`)
    pub const PREFIX: &'static [u8] = PERMISSION_SEED;

    pub fn find_pda(permissioned_account: &compat::Pubkey) -> (compat::Pubkey, u8) {
        compat::Pubkey::find_program_address(
            &[PERMISSION_SEED, permissioned_account.as_ref()],
            &PERMISSION_PROGRAM_ID,
        )
    }
}
