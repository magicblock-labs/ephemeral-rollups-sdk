use dlp_api::pda::{
    delegate_buffer_pda_from_delegated_account_and_owner_program,
    delegation_metadata_pda_from_delegated_account, delegation_record_pda_from_delegated_account,
};

use crate::{
    consts::ESPL_TOKEN_PROGRAM_ID,
    cpi::DELEGATION_PROGRAM_ID,
    solana_compat::solana::{system_program, AccountMeta, Instruction, Pubkey},
    spl::{
        find_lamports_pda, find_rent_pda, find_transfer_queue_refill_state,
        EphemeralSplDiscriminator,
    },
};

pub struct ProcessPendingTransferQueueRefillBuilder {
    pub queue: Pubkey,
}

impl ProcessPendingTransferQueueRefillBuilder {
    #[inline(always)]
    pub fn instruction(&self) -> Instruction {
        let (rent_pda, _rent_bump) = find_rent_pda();
        let queue_bytes = self.queue.to_bytes();
        let (refill_state, _refill_state_bump) = find_transfer_queue_refill_state(&self.queue);
        let (lamports_pda, _lamports_bump) =
            find_lamports_pda(&rent_pda, &self.queue, &queue_bytes);
        let delegation_buffer = delegate_buffer_pda_from_delegated_account_and_owner_program(
            &lamports_pda,
            &ESPL_TOKEN_PROGRAM_ID,
        );
        let delegation_record = delegation_record_pda_from_delegated_account(&lamports_pda);
        let delegation_metadata = delegation_metadata_pda_from_delegated_account(&lamports_pda);
        let queue_delegation_record = delegation_record_pda_from_delegated_account(&self.queue);

        Instruction {
            program_id: ESPL_TOKEN_PROGRAM_ID,
            accounts: vec![
                AccountMeta::new(refill_state, false),
                AccountMeta::new(self.queue, false),
                AccountMeta::new(rent_pda, false),
                AccountMeta::new(lamports_pda, false),
                AccountMeta::new_readonly(ESPL_TOKEN_PROGRAM_ID, false),
                AccountMeta::new(delegation_buffer, false),
                AccountMeta::new(delegation_record, false),
                AccountMeta::new(delegation_metadata, false),
                AccountMeta::new_readonly(DELEGATION_PROGRAM_ID, false),
                AccountMeta::new_readonly(system_program::id(), false),
                AccountMeta::new_readonly(queue_delegation_record, false),
            ],
            data: vec![EphemeralSplDiscriminator::ProcessPendingTransferQueueRefill as u8],
        }
    }
}
