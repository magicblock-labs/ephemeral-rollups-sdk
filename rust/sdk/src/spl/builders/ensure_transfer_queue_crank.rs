use crate::{
    consts::{ESPL_TOKEN_PROGRAM_ID, MAGIC_CONTEXT_ID, MAGIC_PROGRAM_ID},
    solana_compat::solana::{AccountMeta, Instruction, Pubkey},
    spl::EphemeralSplDiscriminator,
};

pub struct EnsureTransferQueueCrankBuilder {
    pub payer: Pubkey,
    pub queue: Pubkey,
    pub magic_fee_vault: Pubkey,
    pub magic_context: Pubkey,
    pub magic_program: Pubkey,
}

impl EnsureTransferQueueCrankBuilder {
    #[inline(always)]
    pub fn instruction(&self) -> Instruction {
        Instruction {
            program_id: ESPL_TOKEN_PROGRAM_ID,
            accounts: vec![
                AccountMeta::new(self.payer, true),
                AccountMeta::new(self.queue, false),
                AccountMeta::new(self.magic_fee_vault, false),
                AccountMeta::new(self.magic_context, false),
                AccountMeta::new_readonly(self.magic_program, false),
            ],
            data: vec![EphemeralSplDiscriminator::EnsureTransferQueueCrank as u8],
        }
    }
}

impl Default for EnsureTransferQueueCrankBuilder {
    fn default() -> Self {
        Self {
            payer: Pubkey::default(),
            queue: Pubkey::default(),
            magic_fee_vault: Pubkey::default(),
            magic_context: MAGIC_CONTEXT_ID,
            magic_program: MAGIC_PROGRAM_ID,
        }
    }
}
