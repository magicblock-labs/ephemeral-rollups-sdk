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

pub struct DelegateCompressedArgs<'a> {
    /// The proof of the account data
    pub validity_proof: CdpValidityProof,
    /// Account meta
    pub account_meta: CdpCompressedAccountMeta,
    /// Owner program id
    pub owner_program_id: &'a Address,
    /// Validator
    pub validator: &'a Address,
    /// Account data before delegation
    pub account_data: &'a [u8],
    /// PDA seeds
    pub borsh_pda_seeds: &'a [u8],
    /// Bump
    pub bump: u8,
}

impl<'a> DelegateCompressedArgs<'a> {
    pub fn try_write_to(&self, data: &mut [u8]) -> Result<usize, ProgramError> {
        let proof_len = if self.validity_proof.0.is_some() {
            129
        } else {
            1
        };
        if data.len()
            < proof_len + 4 + 64 + self.account_data.len() + self.borsh_pda_seeds.len() + 1
        {
            return Err(ProgramError::InvalidArgument);
        }

        let mut offset = 0;
        data[offset] = self.validity_proof.0.is_some() as u8;
        offset += 1;
        if let Some(proof) = self.validity_proof.0 {
            data[offset..offset + 128].copy_from_slice(&proof);
            offset += 128;
        }

        data[offset..offset + 4].copy_from_slice(&self.account_meta.0);
        offset += 4;
        data[offset..offset + 32].copy_from_slice(self.owner_program_id.as_ref());
        offset += 32;
        data[offset..offset + 32].copy_from_slice(self.validator.as_ref());
        offset += 32;
        data[offset..offset + 4].copy_from_slice(&(self.account_data.len() as u32).to_le_bytes());
        offset += 4;
        data[offset..offset + self.account_data.len()].copy_from_slice(self.account_data);
        offset += self.account_data.len();
        data[offset..offset + 4]
            .copy_from_slice(&(self.borsh_pda_seeds.len() as u32).to_le_bytes());
        offset += 4;
        data[offset..offset + self.borsh_pda_seeds.len()].copy_from_slice(self.borsh_pda_seeds);
        offset += self.borsh_pda_seeds.len();
        data[offset] = self.bump;
        offset += 1;

        Ok(offset)
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
    pub fn invoke(&self) -> ProgramResult {
        self.invoke_signed(&[])
    }

    pub fn invoke_signed(&self, signer_seeds: &[Signer<'_, '_>]) -> ProgramResult {
        const LIGHT_ACCOUNTS: usize = 8;
        const TOTAL_ACCOUNTS: usize = 2 + LIGHT_ACCOUNTS;
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
                &self.payer
            } else if i == 1 {
                &self.delegated_account
            } else {
                &self.remaining_accounts[i - 2]
            }
        });

        let mut data =
            [0u8; 8 + 129 + 64 + MAX_ACCOUNT_DATA_SIZE + 1 + MAX_SEEDS * (4 + MAX_SEED_LEN)];
        data[..8].copy_from_slice(&DELEGATE_COMPRESSED_DISCRIMINATOR);
        let args_len = self.args.try_write_to(&mut data[8..])?;
        let total_len = 8 + args_len;

        let ix = InstructionView {
            program_id: self.compressed_delegation_program.address(),
            accounts: &ix_accounts,
            data: &data[..total_len],
        };
        invoke_signed(&ix, &account_views, signer_seeds)
    }
}
