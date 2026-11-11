//! CPI helpers for Magic accounts.
//!
//! Magic accounts are zero-balance accounts that exist only in a Magic context.
//! Rent is paid by a sponsor account at 32 lamports/byte—109x cheaper than
//! Solana's base rent.
//!
//! # Signing Requirements
//!
//! - **Sponsor**: Must always be a signer (all operations).
//! - **Magic account**: Must be a signer only on `create` (prevents pubkey squatting).
//!   Not required to sign on `resize` or `close`.
//!
//! For PDA accounts, provide signers via [`MagicAccount::with_signers`] so the
//! CPI can sign on their behalf using `invoke_signed`. Oncurve accounts must have
//! signed the original transaction.
//!
//! # Example
//!
//! ```ignore
//! use ephemeral_rollups_pinocchio::magic_accounts::MagicAccount;
//!
//! // Create: both sponsor and Magic account are PDAs - provide seeds for both
//! MagicAccount::new(&ctx.sponsor, &ctx.magic, &ctx.vault, &ctx.magic_program)
//!     .with_signers(&[&sponsor_signer, &magic_signer])
//!     .create(1000)?;
//!
//! // Resize/Close: only sponsor needs to sign - only sponsor seeds needed
//! MagicAccount::new(&ctx.sponsor, &ctx.magic, &ctx.vault, &ctx.magic_program)
//!     .with_signers(&[&sponsor_signer])
//!     .resize(2000)?;
//!
//! // Create with oncurve sponsor (signed tx), Magic account is a PDA
//! MagicAccount::new(&ctx.sponsor, &ctx.magic, &ctx.vault, &ctx.magic_program)
//!     .with_signers(&[&magic_signer])
//!     .create(1000)?;
//!
//! // Resize/Close with oncurve sponsor - no seeds needed
//! MagicAccount::new(&ctx.sponsor, &ctx.magic, &ctx.vault, &ctx.magic_program)
//!     .resize(2000)?;
//! ```

use pinocchio::{
    cpi::{invoke_signed, Signer},
    instruction::{InstructionAccount, InstructionView},
    AccountView, ProgramResult,
};

/// Redefined constants to avoid importing the std magicblock-magic-program-api crate.
const ACCOUNT_OVERHEAD: u32 = 60;
const MAGIC_ACCOUNT_RENT_PER_BYTE: u64 = 32;

const CREATE_MAGIC_ACCOUNT_DISCRIMINATOR: u32 = 12;
const RESIZE_MAGIC_ACCOUNT_DISCRIMINATOR: u32 = 13;
const CLOSE_MAGIC_ACCOUNT_DISCRIMINATOR: u32 = 14;

// -----------------
// Utility Functions
// -----------------

/// Calculates rent for a Magic account.
///
/// Rent includes both the data length and the 60-byte account overhead.
///
/// # Example
///
/// ```
/// use ephemeral_rollups_pinocchio::magic_accounts::rent;
///
/// let cost = rent(1000); // Cost for 1KB of data
/// assert_eq!(cost, (1000 + 60) * 32);
/// ```
#[inline]
pub const fn rent(data_len: u32) -> u64 {
    (data_len as u64 + ACCOUNT_OVERHEAD as u64) * MAGIC_ACCOUNT_RENT_PER_BYTE
}

// -----------------
// Builder
// -----------------

/// Builder for Magic account CPI operations.
///
/// Use [`crate::consts::MAGIC_ACCOUNT_VAULT_ID`] when setting up your account structs.
pub struct MagicAccount<'a> {
    sponsor: &'a AccountView,
    magic: &'a AccountView,
    vault: &'a AccountView,
    magic_program: &'a AccountView,
    signer_seeds: &'a [Signer<'a, 'a>],
}

impl<'a> MagicAccount<'a> {
    /// Creates a new builder with the required accounts.
    ///
    /// # Arguments
    ///
    /// * `sponsor` - Account paying rent (must be signer for all operations)
    /// * `magic` - Account to create/modify (must be signer only on create)
    /// * `vault` - Rent vault ([`crate::consts::MAGIC_ACCOUNT_VAULT_ID`])
    /// * `magic_program` - Magic program ([`crate::consts::MAGIC_PROGRAM_ID`])
    pub fn new(
        sponsor: &'a AccountView,
        magic: &'a AccountView,
        vault: &'a AccountView,
        magic_program: &'a AccountView,
    ) -> Self {
        Self {
            sponsor,
            magic,
            vault,
            magic_program,
            signer_seeds: &[],
        }
    }

    /// Sets signer seeds for PDA signing via `invoke_signed`.
    ///
    /// Provide one signer per PDA account (sponsor and/or Magic account).
    /// Oncurve accounts that signed the original transaction don't need seeds.
    pub fn with_signers(mut self, seeds: &'a [Signer<'a, 'a>]) -> Self {
        self.signer_seeds = seeds;
        self
    }

    /// Creates a new Magic account.
    ///
    /// The account will be owned by the calling program (inferred from CPI context).
    /// Rent is transferred from sponsor to vault.
    ///
    /// **Note:** The Magic account must be a signer to prevent pubkey squatting.
    /// Provide signers via [`Self::with_signers`] if it or the sponsor is a PDA.
    pub fn create(&self, data_len: u32) -> ProgramResult {
        let mut data = [0_u8; 8];
        data[0..4].copy_from_slice(&CREATE_MAGIC_ACCOUNT_DISCRIMINATOR.to_le_bytes());
        data[4..8].copy_from_slice(&data_len.to_le_bytes());
        self.invoke(
            &data, true, // the Magic account must sign on create
        )
    }

    /// Resizes an existing Magic account.
    ///
    /// Growing: sponsor pays additional rent to vault.
    /// Shrinking: vault refunds excess rent to sponsor.
    pub fn resize(&self, new_data_len: u32) -> ProgramResult {
        let mut data = [0_u8; 8];
        data[0..4].copy_from_slice(&RESIZE_MAGIC_ACCOUNT_DISCRIMINATOR.to_le_bytes());
        data[4..8].copy_from_slice(&new_data_len.to_le_bytes());
        self.invoke(&data, false)
    }

    /// Closes a Magic account.
    ///
    /// All rent is refunded from vault to sponsor.
    pub fn close(&self) -> ProgramResult {
        self.invoke(&CLOSE_MAGIC_ACCOUNT_DISCRIMINATOR.to_le_bytes(), false)
    }

    fn invoke(&self, data: &[u8], magic_is_signer: bool) -> ProgramResult {
        let ix = InstructionView {
            program_id: self.magic_program.address(),
            data,
            accounts: &[
                InstructionAccount::writable_signer(self.sponsor.address()),
                InstructionAccount::new(self.magic.address(), true, magic_is_signer),
                InstructionAccount::writable(self.vault.address()),
            ],
        };

        invoke_signed(
            &ix,
            &[self.sponsor, self.magic, self.vault],
            self.signer_seeds,
        )
    }
}

/// Legacy name for [`MagicAccount`].
#[deprecated(note = "use `MagicAccount` instead")]
pub use self::MagicAccount as EphemeralAccount;

#[cfg(test)]
mod tests {
    use super::{
        CLOSE_MAGIC_ACCOUNT_DISCRIMINATOR, CREATE_MAGIC_ACCOUNT_DISCRIMINATOR,
        RESIZE_MAGIC_ACCOUNT_DISCRIMINATOR,
    };

    #[test]
    fn magic_account_discriminators_keep_the_existing_wire_values() {
        assert_eq!(CREATE_MAGIC_ACCOUNT_DISCRIMINATOR, 12);
        assert_eq!(RESIZE_MAGIC_ACCOUNT_DISCRIMINATOR, 13);
        assert_eq!(CLOSE_MAGIC_ACCOUNT_DISCRIMINATOR, 14);
    }
}
