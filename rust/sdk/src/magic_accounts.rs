//! CPI helpers for the dedicated Magic Account program on Validator v2.
//!
//! The existing [`crate::ephemeral_accounts`] helpers remain available for the
//! legacy instruction route used by Validator v1 and accepted by Validator v2.

use crate::{
    compat::{self, AsModern, Compat, Modern},
    consts::MAGIC_ACCOUNT_PROGRAM_ID,
};
use magicblock_magic_program_api_v2::{
    instruction::MagicAccountInstruction, MAGIC_ACCOUNT_RENT_PER_BYTE,
};
use solana_program::{
    instruction::{AccountMeta, Instruction},
    program::invoke_signed,
};

const ACCOUNT_OVERHEAD: u32 = 60;

/// Calculates the rent for a Magic account, including the 60-byte account overhead.
#[inline]
pub const fn rent(data_len: u32) -> u64 {
    (data_len as u64 + ACCOUNT_OVERHEAD as u64) * MAGIC_ACCOUNT_RENT_PER_BYTE
}

/// Builder for CPI operations targeting the dedicated Magic Account program.
///
/// The program is Validator-v2-only. Existing [`crate::ephemeral_accounts::EphemeralAccount`]
/// helpers retain the legacy route for Validator v1.
pub struct MagicAccount<'a, 'info> {
    sponsor: &'a compat::AccountInfo<'info>,
    magic: &'a compat::AccountInfo<'info>,
    vault: &'a compat::AccountInfo<'info>,
    signer_seeds: &'a [&'a [&'a [u8]]],
}

impl<'a, 'info> MagicAccount<'a, 'info> {
    /// Creates a builder with the sponsor, Magic account, and rent vault.
    pub fn new(
        sponsor: &'a compat::AccountInfo<'info>,
        magic: &'a compat::AccountInfo<'info>,
        vault: &'a compat::AccountInfo<'info>,
    ) -> Self {
        Self {
            sponsor,
            magic,
            vault,
            signer_seeds: &[],
        }
    }

    /// Sets seeds used to sign for PDA sponsor and/or Magic accounts.
    pub fn with_signer_seeds(mut self, seeds: &'a [&'a [&'a [u8]]]) -> Self {
        self.signer_seeds = seeds;
        self
    }

    /// Creates a Magic account owned by the calling program.
    pub fn create(&self, data_len: u32) -> compat::ProgramResult {
        self.invoke(
            MagicAccountInstruction::CreateMagicAccount { data_len },
            true,
        )
    }

    /// Resizes an existing Magic account.
    pub fn resize(&self, new_data_len: u32) -> compat::ProgramResult {
        self.invoke(
            MagicAccountInstruction::ResizeMagicAccount { new_data_len },
            false,
        )
    }

    /// Closes a Magic account and refunds its rent to the sponsor.
    pub fn close(&self) -> compat::ProgramResult {
        self.invoke(MagicAccountInstruction::CloseMagicAccount, false)
    }

    fn invoke(
        &self,
        instruction: MagicAccountInstruction,
        magic_is_signer: bool,
    ) -> compat::ProgramResult {
        let instruction = Instruction::new_with_bincode(
            *MAGIC_ACCOUNT_PROGRAM_ID.as_modern(),
            &instruction,
            vec![
                AccountMeta::new(*self.sponsor.key.as_modern(), true),
                AccountMeta::new(*self.magic.key.as_modern(), magic_is_signer),
                AccountMeta::new(*self.vault.key.as_modern(), false),
            ],
        );

        invoke_signed(
            &instruction,
            &[
                self.sponsor.modern(),
                self.magic.modern(),
                self.vault.modern(),
            ],
            self.signer_seeds,
        )
        .compat()
    }
}
