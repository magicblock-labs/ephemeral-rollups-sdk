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
//! For PDA accounts, provide seeds via [`MagicAccount::with_signer_seeds`] so the
//! CPI can sign on their behalf using `invoke_signed`. Oncurve accounts must have
//! signed the original transaction.
//!
//! # Example
//!
//! ```ignore
//! use ephemeral_rollups_sdk::magic_accounts::MagicAccount;
//!
//! // Create: both sponsor and Magic account are PDAs - provide seeds for both
//! MagicAccount::new(&ctx.sponsor, &ctx.magic, &ctx.vault)
//!     .with_signer_seeds(&[&sponsor_seeds, &magic_seeds])
//!     .create(1000)?;
//!
//! // Resize/Close: only sponsor needs to sign - only sponsor seeds needed
//! MagicAccount::new(&ctx.sponsor, &ctx.magic, &ctx.vault)
//!     .with_signer_seeds(&[&sponsor_seeds])
//!     .resize(2000)?;
//!
//! // Create with oncurve sponsor (signed tx), Magic account is a PDA
//! MagicAccount::new(&ctx.sponsor, &ctx.magic, &ctx.vault)
//!     .with_signer_seeds(&[&magic_seeds])
//!     .create(1000)?;
//!
//! // Resize/Close with oncurve sponsor - no seeds needed
//! MagicAccount::new(&ctx.sponsor, &ctx.magic, &ctx.vault)
//!     .resize(2000)?;
//! ```

use crate::{
    compat::{self, AsModern, Compat, Modern},
    consts::MAGIC_PROGRAM_ID,
};
use magicblock_magic_program_api_unreleased::{
    instruction::MagicBlockInstruction, MAGIC_ACCOUNT_RENT_PER_BYTE,
};
use solana_program::{
    instruction::{AccountMeta, Instruction},
    program::invoke_signed,
};

/// Account overhead in bytes (static account size in accountsdb).
const ACCOUNT_OVERHEAD: u32 = 60;

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
/// use ephemeral_rollups_sdk::magic_accounts::rent;
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
pub struct MagicAccount<'a, 'info> {
    sponsor: &'a compat::AccountInfo<'info>,
    magic: &'a compat::AccountInfo<'info>,
    vault: &'a compat::AccountInfo<'info>,
    signer_seeds: &'a [&'a [&'a [u8]]],
}

impl<'a, 'info> MagicAccount<'a, 'info> {
    /// Creates a new builder with the required accounts.
    ///
    /// # Arguments
    ///
    /// * `sponsor` - Account paying rent (must be signer for all operations)
    /// * `magic` - Account to create/modify (must be signer only on create)
    /// * `vault` - Rent vault ([`crate::consts::MAGIC_ACCOUNT_VAULT_ID`])
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

    /// Sets signer seeds for PDA signing via `invoke_signed`.
    ///
    /// Provide seeds for any PDA accounts (sponsor and/or Magic account).
    /// Oncurve accounts that signed the original transaction don't need seeds.
    pub fn with_signer_seeds(mut self, seeds: &'a [&'a [&'a [u8]]]) -> Self {
        self.signer_seeds = seeds;
        self
    }

    /// Creates a new Magic account.
    ///
    /// The account will be owned by the calling program (inferred from CPI context).
    /// Rent is transferred from sponsor to vault.
    ///
    /// **Note:** The Magic account must be a signer to prevent pubkey squatting.
    /// Provide seeds via [`Self::with_signer_seeds`] if it is a PDA.
    pub fn create(&self, data_len: u32) -> compat::ProgramResult {
        self.invoke(
            MagicBlockInstruction::CreateMagicAccount { data_len },
            true, // the Magic account must sign on create
        )
    }

    /// Resizes an existing Magic account.
    ///
    /// Growing: sponsor pays additional rent to vault.
    /// Shrinking: vault refunds excess rent to sponsor.
    pub fn resize(&self, new_data_len: u32) -> compat::ProgramResult {
        self.invoke(
            MagicBlockInstruction::ResizeMagicAccount { new_data_len },
            false,
        )
    }

    /// Closes a Magic account.
    ///
    /// All rent is refunded from vault to sponsor.
    pub fn close(&self) -> compat::ProgramResult {
        self.invoke(MagicBlockInstruction::CloseMagicAccount, false)
    }

    fn invoke(
        &self,
        instruction: MagicBlockInstruction,
        magic_is_signer: bool,
    ) -> compat::ProgramResult {
        let ix = Instruction::new_with_bincode(
            *MAGIC_PROGRAM_ID.as_modern(),
            &instruction,
            vec![
                AccountMeta::new(*self.sponsor.key.as_modern(), true),
                AccountMeta::new(*self.magic.key.as_modern(), magic_is_signer),
                AccountMeta::new(*self.vault.key.as_modern(), false),
            ],
        );

        invoke_signed(
            &ix,
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

/// Legacy name for [`MagicAccount`].
#[deprecated(note = "use `MagicAccount` instead")]
pub use self::MagicAccount as EphemeralAccount;

#[cfg(test)]
mod tests {
    use magicblock_magic_program_api::instruction::MagicBlockInstruction as LegacyInstruction;
    use magicblock_magic_program_api_unreleased::instruction::MagicBlockInstruction;

    #[test]
    fn magic_account_instructions_keep_the_existing_wire_encoding() {
        let old_create = LegacyInstruction::CreateEphemeralAccount { data_len: 123 };
        let new_create = MagicBlockInstruction::CreateMagicAccount { data_len: 123 };
        assert_eq!(
            bincode::serialize(&old_create).unwrap(),
            bincode::serialize(&new_create).unwrap()
        );

        let old_resize = LegacyInstruction::ResizeEphemeralAccount { new_data_len: 456 };
        let new_resize = MagicBlockInstruction::ResizeMagicAccount { new_data_len: 456 };
        assert_eq!(
            bincode::serialize(&old_resize).unwrap(),
            bincode::serialize(&new_resize).unwrap()
        );

        assert_eq!(
            bincode::serialize(&LegacyInstruction::CloseEphemeralAccount).unwrap(),
            bincode::serialize(&MagicBlockInstruction::CloseMagicAccount).unwrap()
        );
    }

    #[test]
    fn magic_account_vault_address_is_unchanged() {
        assert_eq!(
            magicblock_magic_program_api::EPHEMERAL_VAULT_PUBKEY.to_bytes(),
            magicblock_magic_program_api_unreleased::MAGIC_ACCOUNT_VAULT_PUBKEY.to_bytes()
        );
    }
}
