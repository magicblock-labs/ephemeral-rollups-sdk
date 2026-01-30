use crate::access_control::structs::Member;
use crate::consts::PERMISSION_PROGRAM_ID;
use crate::solana_compat::solana::Pubkey;

#[cfg(feature = "anchor")]
use anchor_lang::prelude::*;

#[cfg(not(feature = "anchor"))]
use borsh::{BorshDeserialize, BorshSerialize};

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
    ///   1. permissioned_account (`Pubkey`)
    pub const PREFIX: &'static [u8] = PERMISSION_SEED;

    pub fn find_pda(permissioned_account: &Pubkey) -> (Pubkey, u8) {
        Pubkey::find_program_address(
            &[PERMISSION_SEED, permissioned_account.as_ref()],
            &PERMISSION_PROGRAM_ID,
        )
    }
}
