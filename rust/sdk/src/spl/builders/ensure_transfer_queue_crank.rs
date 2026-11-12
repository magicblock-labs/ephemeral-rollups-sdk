use crate::{
    consts::{ESPL_TOKEN_PROGRAM_ID, MAGIC_CONTEXT_ID, MAGIC_PROGRAM_ID},
    spl::EphemeralSplDiscriminator,
};

pub struct EnsureTransferQueueCrankBuilder {
    pub payer: solana_program::pubkey::Pubkey,
    pub queue: solana_program::pubkey::Pubkey,
    pub magic_fee_vault: solana_program::pubkey::Pubkey,
    pub magic_context: solana_program::pubkey::Pubkey,
    pub magic_program: solana_program::pubkey::Pubkey,
}

impl EnsureTransferQueueCrankBuilder {
    #[inline(always)]
    pub fn instruction(&self) -> solana_program::instruction::Instruction {
        solana_program::instruction::Instruction {
            program_id: ESPL_TOKEN_PROGRAM_ID,
            accounts: vec![
                solana_program::instruction::AccountMeta::new(self.payer, true),
                solana_program::instruction::AccountMeta::new(self.queue, false),
                solana_program::instruction::AccountMeta::new(self.magic_fee_vault, false),
                solana_program::instruction::AccountMeta::new(self.magic_context, false),
                solana_program::instruction::AccountMeta::new_readonly(self.magic_program, false),
            ],
            data: vec![EphemeralSplDiscriminator::EnsureTransferQueueCrank as u8],
        }
    }
}

impl Default for EnsureTransferQueueCrankBuilder {
    fn default() -> Self {
        Self {
            payer: solana_program::pubkey::Pubkey::default(),
            queue: solana_program::pubkey::Pubkey::default(),
            magic_fee_vault: solana_program::pubkey::Pubkey::default(),
            magic_context: MAGIC_CONTEXT_ID,
            magic_program: MAGIC_PROGRAM_ID,
        }
    }
}
