use dlp_api::pda::{
    delegate_buffer_pda_from_delegated_account_and_owner_program,
    delegation_metadata_pda_from_delegated_account, delegation_record_pda_from_delegated_account,
};

use crate::{
    consts::ESPL_TOKEN_PROGRAM_ID,
    cpi::DELEGATION_PROGRAM_ID,
    solana_compat::solana::{system_program, AccountMeta, Instruction, Pubkey},
    spl::EphemeralSplDiscriminator,
};

pub struct DelegateTransferQueueBuilder {
    pub payer: Pubkey,
    pub queue: Pubkey,
    pub mint: Pubkey,
}

impl DelegateTransferQueueBuilder {
    #[inline(always)]
    pub fn instruction(&self) -> Instruction {
        let delegation_buffer = delegate_buffer_pda_from_delegated_account_and_owner_program(
            &self.queue,
            &ESPL_TOKEN_PROGRAM_ID,
        );
        let delegation_record = delegation_record_pda_from_delegated_account(&self.queue);
        let delegation_metadata = delegation_metadata_pda_from_delegated_account(&self.queue);

        Instruction {
            program_id: ESPL_TOKEN_PROGRAM_ID,
            accounts: vec![
                AccountMeta::new(self.payer, true),
                AccountMeta::new(self.queue, false),
                AccountMeta::new_readonly(self.mint, false),
                AccountMeta::new_readonly(ESPL_TOKEN_PROGRAM_ID, false),
                AccountMeta::new(delegation_buffer, false),
                AccountMeta::new(delegation_record, false),
                AccountMeta::new(delegation_metadata, false),
                AccountMeta::new_readonly(DELEGATION_PROGRAM_ID, false),
                AccountMeta::new_readonly(system_program::id(), false),
            ],
            data: vec![EphemeralSplDiscriminator::DelegateTransferQueue as u8],
        }
    }
}
