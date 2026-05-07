use crate::spl::compat_pda::{
    delegate_buffer_pda_from_delegated_account_and_owner_program,
    delegation_metadata_pda_from_delegated_account, delegation_record_pda_from_delegated_account,
};

use crate::{
    compat, consts::ESPL_TOKEN_PROGRAM_ID, cpi::DELEGATION_PROGRAM_ID,
    spl::EphemeralSplDiscriminator,
};

pub struct DelegateTransferQueueBuilder {
    pub payer: compat::Pubkey,
    pub queue: compat::Pubkey,
    pub mint: compat::Pubkey,
}

impl DelegateTransferQueueBuilder {
    #[inline(always)]
    pub fn instruction(&self) -> compat::Instruction {
        let delegation_buffer = delegate_buffer_pda_from_delegated_account_and_owner_program(
            &self.queue,
            &ESPL_TOKEN_PROGRAM_ID,
        );
        let delegation_record = delegation_record_pda_from_delegated_account(&self.queue);
        let delegation_metadata = delegation_metadata_pda_from_delegated_account(&self.queue);

        compat::Instruction {
            program_id: ESPL_TOKEN_PROGRAM_ID,
            accounts: vec![
                compat::AccountMeta::new(self.payer, true),
                compat::AccountMeta::new(self.queue, false),
                compat::AccountMeta::new_readonly(self.mint, false),
                compat::AccountMeta::new_readonly(ESPL_TOKEN_PROGRAM_ID, false),
                compat::AccountMeta::new(delegation_buffer, false),
                compat::AccountMeta::new(delegation_record, false),
                compat::AccountMeta::new(delegation_metadata, false),
                compat::AccountMeta::new_readonly(DELEGATION_PROGRAM_ID, false),
                compat::AccountMeta::new_readonly(
                    compat::Pubkey::new_from_array(solana_system_interface::program::ID.to_bytes()),
                    false,
                ),
            ],
            data: vec![EphemeralSplDiscriminator::DelegateTransferQueue as u8],
        }
    }
}
