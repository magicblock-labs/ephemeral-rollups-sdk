use crate::access_control::structs::Member;
use crate::consts::PERMISSION_PROGRAM_ID;

use crate::compat::{self, Pubkey};

#[cfg(feature = "anchor-support")]
#[allow(unused_imports)]
use crate::compat::anchor_lang;

#[cfg(feature = "anchor-support")]
use crate::compat::anchor_lang::{AnchorDeserialize, AnchorSerialize};

//#[cfg(feature = "anchor-support")]
#[allow(unused_imports)]
use crate::compat::borsh;

#[cfg(not(feature = "anchor-support"))]
use crate::compat::borsh::{BorshDeserialize, BorshSerialize};

// IMPORTANT: Keep Pubkey unqualified in Anchor IDL-derived structs. Anchor's
// idl-build recognizes bare Pubkey as the native IDL pubkey type, while
// compat::Pubkey is treated as a custom type that must implement IdlBuild.

pub const PERMISSION_SEED: &[u8] = b"permission:";

#[cfg_attr(feature = "anchor-support", derive(AnchorSerialize, AnchorDeserialize))]
#[cfg_attr(
    not(feature = "anchor-support"),
    derive(BorshSerialize, BorshDeserialize)
)]
#[cfg_attr(
    all(not(feature = "anchor-support"), not(feature = "backward-compat")),
    borsh(crate = "crate::compat::borsh")
)]
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
