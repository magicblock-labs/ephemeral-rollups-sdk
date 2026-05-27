use pinocchio::{
    cpi::{invoke_signed, Signer},
    error::ProgramError,
    instruction::{InstructionAccount, InstructionView},
    AccountView, Address, ProgramResult,
};
use solana_address::{MAX_SEEDS, MAX_SEED_LEN};

use crate::compression::{
    CdpCompressedAccountMeta, CdpValidityProof, DELEGATE_COMPRESSED_DISCRIMINATOR,
    MAX_ACCOUNT_DATA_SIZE,
};

pub const DELEGATE_COMPRESSED_MAX_DATA_LEN: usize = 8   // discriminator
    + 129 // validity_proof (tag + proof)
    + core::mem::size_of::<CdpCompressedAccountMeta>()
    + 64  // owner + validator
    + 4   // account_data len
    + MAX_ACCOUNT_DATA_SIZE
    + (4 + MAX_SEEDS * (4 + MAX_SEED_LEN)) // encoded seeds payload
    + 1; // bump

pub struct DelegateCompressedArgs<'a> {
    /// The proof of the account data
    pub validity_proof: CdpValidityProof,
    /// Account meta
    pub account_meta: CdpCompressedAccountMeta,
    /// Owner program id
    pub owner_program_id: &'a Address,
    /// Validator
    pub validator: &'a Address,
    /// Bump
    pub bump: u8,
    /// Account data before delegation
    pub account_data: &'a [u8],
    /// PDA seeds
    pub borsh_pda_seeds: &'a [u8],
}

impl<'a> DelegateCompressedArgs<'a> {
    pub fn try_write_to(&self, data: &mut [u8]) -> Result<usize, ProgramError> {
        let proof_len = if self.validity_proof.0.is_some() {
            129
        } else {
            1
        };
        if data.len()
            < proof_len
                + core::mem::size_of::<CdpCompressedAccountMeta>()
                + 64  // owner_program_id + validator
                + 1   // bump
                + 4   // account_data length prefix
                + self.account_data.len()
                + self.borsh_pda_seeds.len()
        {
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

        data[offset..offset + 2]
            .copy_from_slice(&self.account_meta.tree_info.root_index.to_le_bytes());
        offset += 2;
        data[offset] = self.account_meta.tree_info.prove_by_index as u8;
        offset += 1;
        data[offset] = self.account_meta.tree_info.merkle_tree_pubkey_index;
        offset += 1;
        data[offset] = self.account_meta.tree_info.queue_pubkey_index;
        offset += 1;
        data[offset..offset + 4]
            .copy_from_slice(&self.account_meta.tree_info.leaf_index.to_le_bytes());
        offset += 4;
        data[offset..offset + 32].copy_from_slice(&self.account_meta.address);
        offset += 32;
        data[offset] = self.account_meta.output_state_tree_index;
        offset += 1;

        data[offset..offset + 32].copy_from_slice(self.owner_program_id.as_ref());
        offset += 32;
        data[offset..offset + 32].copy_from_slice(self.validator.as_ref());
        offset += 32;
        data[offset] = self.bump;
        offset += 1;
        data[offset..offset + 4].copy_from_slice(&(self.account_data.len() as u32).to_le_bytes());
        offset += 4;
        data[offset..offset + self.account_data.len()].copy_from_slice(self.account_data);
        offset += self.account_data.len();
        data[offset..offset + self.borsh_pda_seeds.len()].copy_from_slice(self.borsh_pda_seeds);
        offset += self.borsh_pda_seeds.len();

        Ok(offset)
    }

    pub fn try_write_instruction_data(&self, data: &mut [u8]) -> Result<usize, ProgramError> {
        if data.len() < 8 {
            return Err(ProgramError::InvalidArgument);
        }

        data[..8].copy_from_slice(&DELEGATE_COMPRESSED_DISCRIMINATOR);
        let args_len = self.try_write_to(&mut data[8..])?;
        Ok(8 + args_len)
    }
}

pub struct DelegateCompressed<'a> {
    pub payer: &'a AccountView,
    pub delegated_account: &'a AccountView,
    pub compressed_delegation_program: &'a AccountView,
    pub remaining_accounts: &'a [AccountView],
    pub args: DelegateCompressedArgs<'a>,
}

impl<'a> DelegateCompressed<'a> {
    /// Calculate the data length for the delegate compressed instruction
    pub const fn data_len(seeds_len: &[u8], account_data_len: usize) -> usize {
        let mut i = 0;
        let mut total_seed_len = 0;
        while i < seeds_len.len() {
            total_seed_len += 4 + seeds_len[i] as usize;
            i += 1;
        }
        8 + 129
            + core::mem::size_of::<CdpCompressedAccountMeta>()
            + 64
            + 4
            + account_data_len
            + total_seed_len
            + 1
    }

    /// Invoke the instruction with no signer seeds
    /// Uses a default data buffer of size [DELEGATE_COMPRESSED_MAX_DATA_LEN]
    pub fn invoke(&self) -> ProgramResult {
        self.invoke_signed(&[])
    }

    /// Invoke the instruction with a custom data buffer
    pub fn invoke_with_data(&self, data: &mut [u8]) -> ProgramResult {
        self.invoke_signed_with_data(data, &[])
    }

    /// Invoke the instruction with signer seeds.
    /// Uses a default data buffer of size [DELEGATE_COMPRESSED_MAX_DATA_LEN]
    pub fn invoke_signed(&self, signer_seeds: &[Signer<'_, '_>]) -> ProgramResult {
        let mut data = [0u8; DELEGATE_COMPRESSED_MAX_DATA_LEN];
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

        let ix_accounts = core::array::from_fn::<_, TOTAL_ACCOUNTS, _>(|i| {
            if i == 0 {
                InstructionAccount::writable_signer(self.payer.address())
            } else if i == 1 {
                InstructionAccount::readonly_signer(self.delegated_account.address())
            } else {
                InstructionAccount::new(
                    self.remaining_accounts[i - 2].address(),
                    self.remaining_accounts[i - 2].is_writable(),
                    self.remaining_accounts[i - 2].is_signer(),
                )
            }
        });

        let account_views = core::array::from_fn::<_, TOTAL_ACCOUNTS, _>(|i| {
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
