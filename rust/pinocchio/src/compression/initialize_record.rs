use pinocchio::{
    cpi::{invoke_signed, Signer},
    error::ProgramError,
    instruction::{InstructionAccount, InstructionView},
    AccountView, Address, ProgramResult,
};
use solana_address::{MAX_SEEDS, MAX_SEED_LEN};

use crate::compression::{
    CdpPackedAddressTreeInfo, CdpValidityProof, INITIALIZE_COMPRESSED_RECORD_DISCRIMINATOR,
};

pub const INITIALIZE_COMPRESSED_RECORD_MAX_DATA_LEN: usize =
    8 + 129 + 4 + 1 + 32 + 1 + (4 + MAX_SEEDS * (4 + MAX_SEED_LEN));

#[repr(C)]
pub struct InitializeCompressedRecordArgs<'a> {
    /// The proof of the account data
    pub validity_proof: CdpValidityProof,
    /// Address tree info
    pub address_tree_info: CdpPackedAddressTreeInfo,
    /// Output state tree index
    pub output_state_tree_index: u8,
    /// Owner program id
    pub owner_program_id: &'a Address,
    /// Borsh encoded PDA seeds
    pub borsh_pda_seeds: &'a [u8],
    /// Bump
    pub bump: u8,
}

impl<'a> InitializeCompressedRecordArgs<'a> {
    pub fn try_write_to(&self, data: &mut [u8]) -> Result<usize, ProgramError> {
        let proof_len = if self.validity_proof.0.is_some() {
            129
        } else {
            1
        };
        if data.len() < proof_len + 4 + 1 + 32 + self.borsh_pda_seeds.len() + 1 {
            return Err(ProgramError::InvalidArgument);
        }

        let mut offset = 0;
        data[offset] = self.validity_proof.0.is_some() as u8;
        offset += 1;
        if let Some(proof) = self.validity_proof.0 {
            data[offset..offset + 32].copy_from_slice(&proof.a);
            offset += 32;
            data[offset..offset + 64].copy_from_slice(&proof.b);
            offset += 64;
            data[offset..offset + 32].copy_from_slice(&proof.c);
            offset += 32;
        }

        data[offset] = self.address_tree_info.address_merkle_tree_pubkey_index;
        offset += 1;
        data[offset] = self.address_tree_info.address_queue_pubkey_index;
        offset += 1;
        data[offset..offset + 2].copy_from_slice(&self.address_tree_info.root_index.to_le_bytes());
        offset += 2;
        data[offset] = self.output_state_tree_index;
        offset += 1;
        data[offset..offset + 32].copy_from_slice(self.owner_program_id.as_ref());
        offset += 32;
        data[offset..offset + self.borsh_pda_seeds.len()].copy_from_slice(self.borsh_pda_seeds);
        offset += self.borsh_pda_seeds.len();
        data[offset] = self.bump;
        offset += 1;

        Ok(offset)
    }

    pub fn try_write_instruction_data(&self, data: &mut [u8]) -> Result<usize, ProgramError> {
        if data.len() < 8 {
            return Err(ProgramError::InvalidArgument);
        }

        data[..8].copy_from_slice(&INITIALIZE_COMPRESSED_RECORD_DISCRIMINATOR);
        let args_len = self.try_write_to(&mut data[8..])?;
        Ok(8 + args_len)
    }
}

pub struct InitializeCompressedRecord<'a> {
    pub payer: &'a AccountView,
    pub delegated_account: &'a AccountView,
    pub compressed_delegation_program: &'a AccountView,
    pub remaining_accounts: &'a [AccountView],
    pub args: InitializeCompressedRecordArgs<'a>,
}

impl<'a> InitializeCompressedRecord<'a> {
    /// Calculate the data length for the initialize compressed record instruction
    ///
    /// # Arguments
    ///
    /// * `seeds_len` - An array of the length of each seed
    ///
    /// # Returns
    ///
    /// The data length for the initialize compressed record instruction data array
    ///
    /// # Examples
    ///
    /// ```ignore
    /// let seeds = ["counter", "1234567890"];
    /// let seeds_len = [7, 10];
    /// let data_len = InitializeCompressedRecord::data_len(&seeds_len);
    /// InitializeCompressedRecord { .. }.invoke_with_data(&mut data)?;
    /// ```
    pub const fn data_len(seeds_len: &[u8]) -> usize {
        let mut i = 0;
        let mut total_seed_len = 0;
        while i < seeds_len.len() {
            total_seed_len += 4 + seeds_len[i] as usize;
            i += 1;
        }
        8 + 129 + 4 + 1 + 32 + 1 + 4 + total_seed_len
    }

    /// Invoke the instruction with no signer seeds
    /// Uses a default data buffer of size [INITIALIZE_COMPRESSED_RECORD_MAX_DATA_LEN]
    pub fn invoke(&self) -> ProgramResult {
        self.invoke_signed(&[])
    }

    /// Invoke the instruction with a custom data buffer
    pub fn invoke_with_data(&self, data: &mut [u8]) -> ProgramResult {
        self.invoke_signed_with_data(data, &[])
    }

    /// Invoke the instruction with signer seeds.
    /// Uses a default data buffer of size [INITIALIZE_COMPRESSED_RECORD_MAX_DATA_LEN]
    pub fn invoke_signed(&self, signer_seeds: &[Signer<'_, '_>]) -> ProgramResult {
        let mut data = [0u8; INITIALIZE_COMPRESSED_RECORD_MAX_DATA_LEN];
        self.invoke_signed_with_data(&mut data, signer_seeds)
    }

    /// Invoke the instruction with a custom data buffer and signer seeds
    pub fn invoke_signed_with_data(
        &self,
        data: &mut [u8],
        signer_seeds: &[Signer<'_, '_>],
    ) -> ProgramResult {
        const LIGHT_ACCOUNTS: usize = 8;
        const TOTAL_ACCOUNTS: usize = 2 + LIGHT_ACCOUNTS;

        if self.remaining_accounts.len() < LIGHT_ACCOUNTS {
            return Err(ProgramError::NotEnoughAccountKeys);
        }

        // 2 + 8 Accounts from Light
        let ix_accounts: [InstructionAccount<'_>; TOTAL_ACCOUNTS] = core::array::from_fn(|i| {
            if i == 0 {
                InstructionAccount::writable_signer(self.payer.address())
            } else if i == 1 {
                InstructionAccount::readonly_signer(self.delegated_account.address())
            } else {
                InstructionAccount {
                    address: self.remaining_accounts[i - 2].address(),
                    is_writable: self.remaining_accounts[i - 2].is_writable(),
                    is_signer: self.remaining_accounts[i - 2].is_signer(),
                }
            }
        });

        let account_views: [&AccountView; TOTAL_ACCOUNTS] = core::array::from_fn(|i| {
            if i == 0 {
                self.payer
            } else if i == 1 {
                self.delegated_account
            } else {
                &self.remaining_accounts[i - 2]
            }
        });

        let total_len = self.args.try_write_instruction_data(data)?;
        let ix = InstructionView {
            program_id: self.compressed_delegation_program.address(),
            accounts: &ix_accounts,
            data: &data[..total_len],
        };

        invoke_signed(&ix, &account_views, signer_seeds)
    }
}
