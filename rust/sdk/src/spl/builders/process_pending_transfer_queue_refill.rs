use crate::pda::{
    delegate_buffer_pda_from_delegated_account_and_owner_program,
    delegation_metadata_pda_from_delegated_account, delegation_record_pda_from_delegated_account,
};

use crate::{
    consts::ESPL_TOKEN_PROGRAM_ID,
    cpi::DELEGATION_PROGRAM_ID,
    spl::{
        find_lamports_pda, find_rent_pda, find_transfer_queue_refill_state,
        EphemeralSplDiscriminator,
    },
};

pub struct ProcessPendingTransferQueueRefillBuilder {
    pub queue: solana_program::pubkey::Pubkey,
}

impl ProcessPendingTransferQueueRefillBuilder {
    #[inline(always)]
    pub fn instruction(&self) -> solana_program::instruction::Instruction {
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

        solana_program::instruction::Instruction {
            program_id: ESPL_TOKEN_PROGRAM_ID,
            accounts: vec![
                solana_program::instruction::AccountMeta::new(refill_state, false),
                solana_program::instruction::AccountMeta::new(self.queue, false),
                solana_program::instruction::AccountMeta::new(rent_pda, false),
                solana_program::instruction::AccountMeta::new(lamports_pda, false),
                solana_program::instruction::AccountMeta::new_readonly(
                    ESPL_TOKEN_PROGRAM_ID,
                    false,
                ),
                solana_program::instruction::AccountMeta::new(delegation_buffer, false),
                solana_program::instruction::AccountMeta::new(delegation_record, false),
                solana_program::instruction::AccountMeta::new(delegation_metadata, false),
                solana_program::instruction::AccountMeta::new_readonly(
                    DELEGATION_PROGRAM_ID,
                    false,
                ),
                solana_program::instruction::AccountMeta::new_readonly(
                    solana_program::pubkey::Pubkey::new_from_array(
                        solana_system_interface::program::ID.to_bytes(),
                    ),
                    false,
                ),
                solana_program::instruction::AccountMeta::new_readonly(
                    queue_delegation_record,
                    false,
                ),
            ],
            data: vec![EphemeralSplDiscriminator::ProcessPendingTransferQueueRefill as u8],
        }
    }
}
