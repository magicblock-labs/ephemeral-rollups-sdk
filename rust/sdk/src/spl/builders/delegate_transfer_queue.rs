use crate::pda::{
    delegate_buffer_pda_from_delegated_account_and_owner_program,
    delegation_metadata_pda_from_delegated_account, delegation_record_pda_from_delegated_account,
};

use crate::{
    consts::ESPL_TOKEN_PROGRAM_ID, cpi::DELEGATION_PROGRAM_ID, spl::EphemeralSplDiscriminator,
};

pub struct DelegateTransferQueueBuilder {
    pub payer: solana_program::pubkey::Pubkey,
    pub queue: solana_program::pubkey::Pubkey,
    pub mint: solana_program::pubkey::Pubkey,
}

impl DelegateTransferQueueBuilder {
    #[inline(always)]
    pub fn instruction(&self) -> solana_program::instruction::Instruction {
        let delegation_buffer = delegate_buffer_pda_from_delegated_account_and_owner_program(
            &self.queue,
            &ESPL_TOKEN_PROGRAM_ID,
        );
        let delegation_record = delegation_record_pda_from_delegated_account(&self.queue);
        let delegation_metadata = delegation_metadata_pda_from_delegated_account(&self.queue);

        solana_program::instruction::Instruction {
            program_id: ESPL_TOKEN_PROGRAM_ID,
            accounts: vec![
                solana_program::instruction::AccountMeta::new(self.payer, true),
                solana_program::instruction::AccountMeta::new(self.queue, false),
                solana_program::instruction::AccountMeta::new_readonly(self.mint, false),
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
            ],
            data: vec![EphemeralSplDiscriminator::DelegateTransferQueue as u8],
        }
    }
}
