use crate::solana_compat::solana::{system_program, AccountMeta, Pubkey};
use dlp::pda::{
    delegate_buffer_pda_from_delegated_account_and_owner_program,
    delegation_metadata_pda_from_delegated_account, delegation_record_pda_from_delegated_account,
};

use crate::consts::DELEGATION_PROGRAM_ID;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DelegateAccounts {
    pub delegated_account: Pubkey,
    pub delegate_buffer: Pubkey,
    pub delegation_record: Pubkey,
    pub delegation_metadata: Pubkey,
    pub owner_program: Pubkey,
    pub delegation_program: Pubkey,
    pub system_program: Pubkey,
}

impl DelegateAccounts {
    pub fn new(delegated_account: Pubkey, owner_program: Pubkey) -> Self {
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
            delegate_buffer: delegate_buffer.to_bytes().into(),
            delegation_record: delegation_record.to_bytes().into(),
            delegation_metadata: delegation_metadata.to_bytes().into(),
            owner_program,
            delegation_program: DELEGATION_PROGRAM_ID.to_bytes().into(),
            system_program: system_program::ID.to_bytes().into(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DelegateAccountMetas {
    pub delegated_account: AccountMeta,
    pub owner_program: AccountMeta,
    pub delegate_buffer: AccountMeta,
    pub delegation_record: AccountMeta,
    pub delegation_metadata: AccountMeta,
    pub delegation_program: AccountMeta,
    pub system_program: AccountMeta,
}

impl From<DelegateAccounts> for DelegateAccountMetas {
    fn from(accounts: DelegateAccounts) -> Self {
        Self {
            delegated_account: AccountMeta::new(accounts.delegated_account, false),
            owner_program: AccountMeta::new_readonly(accounts.owner_program, false),
            delegate_buffer: AccountMeta::new(accounts.delegate_buffer, false),
            delegation_record: AccountMeta::new(accounts.delegation_record, false),
            delegation_metadata: AccountMeta::new(accounts.delegation_metadata, false),
            delegation_program: AccountMeta::new_readonly(accounts.delegation_program, false),
            system_program: AccountMeta::new_readonly(accounts.system_program, false),
        }
    }
}
