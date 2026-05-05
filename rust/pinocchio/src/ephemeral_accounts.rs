//! CPI helpers for ephemeral accounts.
//!
//! Ephemeral accounts are zero-balance accounts that exist only in the ephemeral
//! rollup. Rent is paid by a sponsor account at 32 lamports/byte—109x cheaper
//! than Solana's base rent.
//!
//! # Signing Requirements
//!
//! - **Sponsor**: Must always be a signer (all operations).
//! - **Ephemeral**: Must be a signer only on `create` (prevents pubkey squatting).
//!   Not required to sign on `resize` or `close`.
//!
//! For PDA accounts, provide seeds via [`EphemeralAccount::with_signer_seeds`] so the
//! CPI can sign on their behalf using `invoke_signed`. Oncurve accounts must have
//! signed the original transaction.
//!
//! # Example
//!
//! ```ignore
//! use ephemeral_rollups_pinocchio::ephemeral_accounts::EphemeralAccount;
//!
//! // Create: both sponsor and ephemeral are PDAs - provide seeds for both
//! EphemeralAccount::new(&ctx.sponsor, &ctx.ephemeral, &ctx.vault)
//!     .with_signer_seeds(&[&sponsor_seeds, &ephemeral_seeds])
//!     .create(1000)?;
//!
//! // Resize/Close: only sponsor needs to sign - only sponsor seeds needed
//! EphemeralAccount::new(&ctx.sponsor, &ctx.ephemeral, &ctx.vault)
//!     .with_signer_seeds(&[&sponsor_seeds])
//!     .resize(2000)?;
//!
//! // Create with oncurve sponsor (signed tx), ephemeral is PDA
//! EphemeralAccount::new(&ctx.sponsor, &ctx.ephemeral, &ctx.vault)
//!     .with_signer_seeds(&[&ephemeral_seeds])
//!     .create(1000)?;
//!
//! // Resize/Close with oncurve sponsor - no seeds needed
//! EphemeralAccount::new(&ctx.sponsor, &ctx.ephemeral, &ctx.vault)
//!     .resize(2000)?;
//! ```

use pinocchio::{
    cpi::{invoke_signed, Seed, Signer},
    instruction::{InstructionAccount, InstructionView},
    AccountView, ProgramResult,
};

/// Redefined constants to avoid importing the std magicblock-magic-program-api crate.
const ACCOUNT_OVERHEAD: u32 = 60;
const EPHEMERAL_RENT_PER_BYTE: u64 = 32;

const CREATE_EPHEMERAL_ACCOUNT_DISCRIMINATOR: u8 = 12;
const RESIZE_EPHEMERAL_ACCOUNT_DISCRIMINATOR: u8 = 13;
const CLOSE_EPHEMERAL_ACCOUNT_DISCRIMINATOR: u8 = 14;

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
pub const fn rent(data_len: u32) -> u64 {
    (data_len as u64 + ACCOUNT_OVERHEAD as u64) * EPHEMERAL_RENT_PER_BYTE
}

// -----------------
// Builder
// -----------------

/// Builder for ephemeral account CPI operations.
///
/// Use [`crate::consts::EPHEMERAL_VAULT_ID`] when setting up your account structs.
pub struct EphemeralAccount<'a> {
    sponsor: &'a AccountView,
    ephemeral: &'a AccountView,
    vault: &'a AccountView,
    magic_proggram: &'a AccountView,
    signer_seeds: &'a [Seed<'a>],
}

impl<'a> EphemeralAccount<'a> {
    /// Creates a new builder with the required accounts.
    ///
    /// # Arguments
    ///
    /// * `sponsor` - Account paying rent (must be signer for all operations)
    /// * `ephemeral` - Account to create/modify (must be signer only on create)
    /// * `vault` - Rent vault ([`crate::consts::EPHEMERAL_VAULT_ID`])
    pub fn new(
        sponsor: &'a AccountView,
        ephemeral: &'a AccountView,
        vault: &'a AccountView,
        magic_proggram: &'a AccountView,
    ) -> Self {
        Self {
            sponsor,
            ephemeral,
            vault,
            magic_proggram,
            signer_seeds: &[],
        }
    }

    /// Sets signer seeds for PDA signing via `invoke_signed`.
    ///
    /// Provide seeds for any PDA accounts (sponsor and/or ephemeral).
    /// Oncurve accounts that signed the original transaction don't need seeds.
    pub fn with_signer_seeds(mut self, seeds: &'a [Seed<'a>]) -> Self {
        self.signer_seeds = seeds;
        self
    }

    /// Creates a new ephemeral account.
    ///
    /// The account will be owned by the calling program (inferred from CPI context).
    /// Rent is transferred from sponsor to vault.
    ///
    /// **Note:** Ephemeral account must be a signer to prevent pubkey squatting.
    /// Provide seeds via [`Self::with_signer_seeds`] if ephemeral is a PDA.
    pub fn create(&self, data_len: u32) -> ProgramResult {
        let mut data = [0_u8; 5];
        data[0] = CREATE_EPHEMERAL_ACCOUNT_DISCRIMINATOR;
        data[1..5].copy_from_slice(&data_len.to_le_bytes());
        self.invoke(
            &data, true, // ephemeral must sign on create
        )
    }

    /// Resizes an existing ephemeral account.
    ///
    /// Growing: sponsor pays additional rent to vault.
    /// Shrinking: vault refunds excess rent to sponsor.
    pub fn resize(&self, new_data_len: u32) -> ProgramResult {
        let mut data = [0_u8; 5];
        data[0] = RESIZE_EPHEMERAL_ACCOUNT_DISCRIMINATOR;
        data[1..5].copy_from_slice(&new_data_len.to_le_bytes());
        self.invoke(&data, false)
    }

    /// Closes an ephemeral account.
    ///
    /// All rent is refunded from vault to sponsor.
    pub fn close(&self) -> ProgramResult {
        self.invoke(&[CLOSE_EPHEMERAL_ACCOUNT_DISCRIMINATOR], false)
    }

    fn invoke(&self, data: &[u8], ephemeral_is_signer: bool) -> ProgramResult {
        let ix = InstructionView {
            program_id: self.magic_proggram.address(),
            data,
            accounts: &[
                InstructionAccount::writable_signer(self.sponsor.address()),
                InstructionAccount::new(self.ephemeral.address(), true, ephemeral_is_signer),
                InstructionAccount::writable(self.vault.address()),
            ],
        };

        let signer = Signer::from(self.signer_seeds);
        invoke_signed(&ix, &[self.sponsor, self.ephemeral, self.vault], &[signer])
    }
}
