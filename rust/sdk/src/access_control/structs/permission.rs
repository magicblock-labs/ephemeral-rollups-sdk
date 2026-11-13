use crate::access_control::structs::Member;
use crate::consts::PERMISSION_PROGRAM_ID;

use solana_program::pubkey::Pubkey;

#[cfg(feature = "anchor")]
#[allow(unused_imports)]
use anchor_lang;

#[cfg(feature = "anchor")]
use anchor_lang::{AnchorDeserialize, AnchorSerialize};

//#[cfg(feature = "anchor")]
#[allow(unused_imports)]
use borsh;

#[cfg(not(feature = "anchor"))]
use borsh::{BorshDeserialize, BorshSerialize};

// IMPORTANT: Keep Pubkey unqualified in Anchor IDL-derived structs. Anchor's
// idl-build recognizes bare Pubkey as the native IDL pubkey type, while
// solana_program::pubkey::Pubkey is treated as a custom type that must implement IdlBuild.

pub const PERMISSION_SEED: &[u8] = b"permission:";

#[cfg_attr(feature = "anchor", derive(AnchorSerialize, AnchorDeserialize))]
#[cfg_attr(not(feature = "anchor"), derive(BorshSerialize, BorshDeserialize))]
#[cfg_attr(not(feature = "anchor"), borsh(crate = "borsh"))]
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
    ///   1. permissioned_account (`solana_program::pubkey::Pubkey`)
    pub const PREFIX: &'static [u8] = PERMISSION_SEED;

    pub fn find_pda(
        permissioned_account: &solana_program::pubkey::Pubkey,
    ) -> (solana_program::pubkey::Pubkey, u8) {
        solana_program::pubkey::Pubkey::find_program_address(
            &[PERMISSION_SEED, permissioned_account.as_ref()],
            &PERMISSION_PROGRAM_ID,
        )
    }
}
