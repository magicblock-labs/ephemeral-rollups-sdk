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
            data[offset..offset + 128].copy_from_slice(&proof);
            offset += 128;
        }

        data[offset..offset + 4].copy_from_slice(&self.address_tree_info.0);
        offset += 4;
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
}

pub struct InitializeCompressedRecord<'a> {
    pub payer: &'a AccountView,
    pub delegated_account: &'a AccountView,
    pub compressed_delegation_program: &'a AccountView,
    pub remaining_accounts: &'a [AccountView],
    pub args: InitializeCompressedRecordArgs<'a>,
}

impl<'a> InitializeCompressedRecord<'a> {
    pub fn invoke(&self) -> ProgramResult {
        self.invoke_signed(&[])
    }

    pub fn invoke_signed(&self, signer_seeds: &[Signer<'_, '_>]) -> ProgramResult {
        const LIGHT_ACCOUNTS: usize = 8;
        const TOTAL_ACCOUNTS: usize = 2 + LIGHT_ACCOUNTS;

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
                &self.payer
            } else if i == 1 {
                &self.delegated_account
            } else {
                &self.remaining_accounts[i - 2]
            }
        });

        let mut data = [0u8; 8 + 129 + 4 + 1 + 32 + 1 + MAX_SEEDS * (4 + MAX_SEED_LEN)];
        data[..8].copy_from_slice(&INITIALIZE_COMPRESSED_RECORD_DISCRIMINATOR);
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
