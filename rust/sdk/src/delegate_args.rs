use crate::compat::{self, Compat};
use dlp_api::pda::{
    delegate_buffer_pda_from_delegated_account_and_owner_program,
    delegation_metadata_pda_from_delegated_account, delegation_record_pda_from_delegated_account,
};

use crate::consts::DELEGATION_PROGRAM_ID;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DelegateAccounts {
    pub delegated_account: compat::Pubkey,
    pub delegate_buffer: compat::Pubkey,
    pub delegation_record: compat::Pubkey,
    pub delegation_metadata: compat::Pubkey,
    pub owner_program: compat::Pubkey,
    pub delegation_program: compat::Pubkey,
    pub system_program: compat::Pubkey,
}

impl DelegateAccounts {
    pub fn new(delegated_account: compat::Pubkey, owner_program: compat::Pubkey) -> Self {
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
            delegate_buffer: delegate_buffer.compat(),
            delegation_record: delegation_record.compat(),
            delegation_metadata: delegation_metadata.compat(),
            owner_program,
            delegation_program: DELEGATION_PROGRAM_ID.compat(),
            system_program: solana_system_interface::program::ID.compat(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DelegateAccountMetas {
    pub delegated_account: compat::AccountMeta,
    pub owner_program: compat::AccountMeta,
    pub delegate_buffer: compat::AccountMeta,
    pub delegation_record: compat::AccountMeta,
    pub delegation_metadata: compat::AccountMeta,
    pub delegation_program: compat::AccountMeta,
    pub system_program: compat::AccountMeta,
}

impl From<DelegateAccounts> for DelegateAccountMetas {
    fn from(accounts: DelegateAccounts) -> Self {
        Self {
            delegated_account: compat::AccountMeta::new(accounts.delegated_account, false),
            owner_program: compat::AccountMeta::new_readonly(accounts.owner_program, false),
            delegate_buffer: compat::AccountMeta::new(accounts.delegate_buffer, false),
            delegation_record: compat::AccountMeta::new(accounts.delegation_record, false),
            delegation_metadata: compat::AccountMeta::new(accounts.delegation_metadata, false),
            delegation_program: compat::AccountMeta::new_readonly(
                accounts.delegation_program,
                false,
            ),
            system_program: compat::AccountMeta::new_readonly(accounts.system_program, false),
        }
    }
}
