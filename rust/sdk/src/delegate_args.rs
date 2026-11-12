use dlp_api::pda::{
    delegate_buffer_pda_from_delegated_account_and_owner_program,
    delegation_metadata_pda_from_delegated_account, delegation_record_pda_from_delegated_account,
};

use crate::consts::DELEGATION_PROGRAM_ID;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DelegateAccounts {
    pub delegated_account: solana_program::pubkey::Pubkey,
    pub delegate_buffer: solana_program::pubkey::Pubkey,
    pub delegation_record: solana_program::pubkey::Pubkey,
    pub delegation_metadata: solana_program::pubkey::Pubkey,
    pub owner_program: solana_program::pubkey::Pubkey,
    pub delegation_program: solana_program::pubkey::Pubkey,
    pub system_program: solana_program::pubkey::Pubkey,
}

impl DelegateAccounts {
    pub fn new(
        delegated_account: solana_program::pubkey::Pubkey,
        owner_program: solana_program::pubkey::Pubkey,
    ) -> Self {
        let delegate_buffer = delegate_buffer_pda_from_delegated_account_and_owner_program(
            &delegated_account.to_bytes().into(),
            &owner_program.to_bytes().into(),
        );
        let delegation_record =
            delegation_record_pda_from_delegated_account(&delegated_account.to_bytes().into());
        let delegation_metadata =
            delegation_metadata_pda_from_delegated_account(&delegated_account.to_bytes().into());
        Self {
            delegated_account,
            delegate_buffer,
            delegation_record,
            delegation_metadata,
            owner_program,
            delegation_program: DELEGATION_PROGRAM_ID,
            system_program: solana_system_interface::program::ID,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DelegateAccountMetas {
    pub delegated_account: solana_program::instruction::AccountMeta,
    pub owner_program: solana_program::instruction::AccountMeta,
    pub delegate_buffer: solana_program::instruction::AccountMeta,
    pub delegation_record: solana_program::instruction::AccountMeta,
    pub delegation_metadata: solana_program::instruction::AccountMeta,
    pub delegation_program: solana_program::instruction::AccountMeta,
    pub system_program: solana_program::instruction::AccountMeta,
}

impl From<DelegateAccounts> for DelegateAccountMetas {
    fn from(accounts: DelegateAccounts) -> Self {
        Self {
            delegated_account: solana_program::instruction::AccountMeta::new(
                accounts.delegated_account,
                false,
            ),
            owner_program: solana_program::instruction::AccountMeta::new_readonly(
                accounts.owner_program,
                false,
            ),
            delegate_buffer: solana_program::instruction::AccountMeta::new(
                accounts.delegate_buffer,
                false,
            ),
            delegation_record: solana_program::instruction::AccountMeta::new(
                accounts.delegation_record,
                false,
            ),
            delegation_metadata: solana_program::instruction::AccountMeta::new(
                accounts.delegation_metadata,
                false,
            ),
            delegation_program: solana_program::instruction::AccountMeta::new_readonly(
                accounts.delegation_program,
                false,
            ),
            system_program: solana_program::instruction::AccountMeta::new_readonly(
                accounts.system_program,
                false,
            ),
        }
    }
}
