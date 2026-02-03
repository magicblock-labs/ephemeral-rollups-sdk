//! CPI helpers for ephemeral accounts.
//!
//! Ephemeral accounts are zero-balance accounts that exist only in the ephemeral
//! rollup. Rent is paid by a sponsor account at 32 lamports/byte—109x cheaper
//! than Solana's base rent.
//!
//! # Signing Requirements
//!
//! Both `sponsor` and `ephemeral` accounts must be signers:
//!
//! - **PDA accounts**: Provide seeds via [`EphemeralAccount::with_signer_seeds`] so the
//!   CPI can sign on their behalf using `invoke_signed`.
//! - **Oncurve accounts**: Must have signed the original transaction. No seeds needed,
//!   but ensure `is_signer` is true on the `AccountInfo`.
//!
//! # Example
//!
//! ```ignore
//! use ephemeral_rollups_sdk::ephemeral_accounts::EphemeralAccount;
//!
//! // Both sponsor and ephemeral are PDAs - provide seeds for both
//! EphemeralAccount::new(&ctx.sponsor, &ctx.ephemeral, &ctx.vault)
//!     .with_signer_seeds(&[&sponsor_seeds, &ephemeral_seeds])
//!     .create(1000)?;
//!
//! // Sponsor is oncurve (signed tx), ephemeral is PDA - only ephemeral seeds needed
//! EphemeralAccount::new(&ctx.sponsor, &ctx.ephemeral, &ctx.vault)
//!     .with_signer_seeds(&[&ephemeral_seeds])
//!     .create(1000)?;
//!
//! // Both are oncurve and signed the transaction - no seeds needed
//! EphemeralAccount::new(&ctx.sponsor, &ctx.ephemeral, &ctx.vault)
//!     .create(1000)?;
//! ```

use crate::{
    consts::MAGIC_PROGRAM_ID,
    solana_compat::solana::{invoke_signed, AccountInfo, AccountMeta, Instruction, ProgramResult},
};
use magicblock_magic_program_api::instruction::MagicBlockInstruction;

pub use magicblock_magic_program_api::EPHEMERAL_RENT_PER_BYTE;

/// Account overhead in bytes (static account size in accountsdb).
const ACCOUNT_OVERHEAD: usize = 60;

// -----------------
// Utility Functions
// -----------------

/// Calculates rent for an ephemeral account.
///
/// Rent includes both the data length and the 60-byte account overhead.
///
/// # Example
///
/// ```
/// use ephemeral_rollups_sdk::ephemeral_accounts::rent;
///
/// let cost = rent(1000); // Cost for 1KB of data
/// assert_eq!(cost, (1000 + 60) * 32);
/// ```
#[inline]
pub const fn rent(data_len: usize) -> u64 {
    (data_len + ACCOUNT_OVERHEAD) as u64 * EPHEMERAL_RENT_PER_BYTE
}

/// Calculates rent difference when resizing an account.
///
/// Returns positive if growing (sponsor pays), negative if shrinking (sponsor receives).
#[inline]
pub const fn rent_delta(old_len: usize, new_len: usize) -> i64 {
    rent(new_len) as i64 - rent(old_len) as i64
}

// -----------------
// Builder
// -----------------

/// Builder for ephemeral account CPI operations.
///
/// Use [`crate::consts::EPHEMERAL_VAULT_ID`] when setting up your account structs.
pub struct EphemeralAccount<'a, 'info> {
    sponsor: &'a AccountInfo<'info>,
    ephemeral: &'a AccountInfo<'info>,
    vault: &'a AccountInfo<'info>,
    signer_seeds: &'a [&'a [&'a [u8]]],
}

impl<'a, 'info> EphemeralAccount<'a, 'info> {
    /// Creates a new builder with the required accounts.
    ///
    /// # Arguments
    ///
    /// * `sponsor` - Account paying rent (must be signer)
    /// * `ephemeral` - Account to create/modify (must be signer)
    /// * `vault` - Rent vault ([`crate::consts::EPHEMERAL_VAULT_ID`])
    pub fn new(
        sponsor: &'a AccountInfo<'info>,
        ephemeral: &'a AccountInfo<'info>,
        vault: &'a AccountInfo<'info>,
    ) -> Self {
        Self {
            sponsor,
            ephemeral,
            vault,
            signer_seeds: &[],
        }
    }

    /// Sets signer seeds for PDA signing via `invoke_signed`.
    ///
    /// Provide seeds for any PDA accounts (sponsor and/or ephemeral).
    /// Oncurve accounts that signed the original transaction don't need seeds.
    pub fn with_signer_seeds(mut self, seeds: &'a [&'a [&'a [u8]]]) -> Self {
        self.signer_seeds = seeds;
        self
    }

    /// Creates a new ephemeral account.
    ///
    /// The account will be owned by the calling program (inferred from CPI context).
    /// Rent is transferred from sponsor to vault.
    pub fn create(&self, data_len: u32) -> ProgramResult {
        self.invoke(MagicBlockInstruction::CreateEphemeralAccount { data_len })
    }

    /// Resizes an existing ephemeral account.
    ///
    /// Growing: sponsor pays additional rent to vault.
    /// Shrinking: vault refunds excess rent to sponsor.
    pub fn resize(&self, new_data_len: u32) -> ProgramResult {
        self.invoke(MagicBlockInstruction::ResizeEphemeralAccount { new_data_len })
    }

    /// Closes an ephemeral account.
    ///
    /// All rent is refunded from vault to sponsor.
    pub fn close(&self) -> ProgramResult {
        self.invoke(MagicBlockInstruction::CloseEphemeralAccount)
    }

    fn invoke(&self, instruction: MagicBlockInstruction) -> ProgramResult {
        let ix = Instruction::new_with_bincode(
            MAGIC_PROGRAM_ID,
            &instruction,
            vec![
                AccountMeta::new(*self.sponsor.key, true),
                AccountMeta::new(*self.ephemeral.key, true),
                AccountMeta::new(*self.vault.key, false),
            ],
        );

        invoke_signed(
            &ix,
            &[
                self.sponsor.clone(),
                self.ephemeral.clone(),
                self.vault.clone(),
            ],
            self.signer_seeds,
        )
    }
}
