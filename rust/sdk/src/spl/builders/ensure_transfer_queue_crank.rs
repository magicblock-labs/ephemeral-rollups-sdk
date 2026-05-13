use crate::{
    compat,
    consts::{ESPL_TOKEN_PROGRAM_ID, MAGIC_CONTEXT_ID, MAGIC_PROGRAM_ID},
    spl::EphemeralSplDiscriminator,
};

pub struct EnsureTransferQueueCrankBuilder {
    pub payer: compat::Pubkey,
    pub queue: compat::Pubkey,
    pub magic_fee_vault: compat::Pubkey,
    pub magic_context: compat::Pubkey,
    pub magic_program: compat::Pubkey,
}

impl EnsureTransferQueueCrankBuilder {
    #[inline(always)]
    pub fn instruction(&self) -> compat::Instruction {
        compat::Instruction {
            program_id: ESPL_TOKEN_PROGRAM_ID,
            accounts: vec![
                compat::AccountMeta::new(self.payer, true),
                compat::AccountMeta::new(self.queue, false),
                compat::AccountMeta::new(self.magic_fee_vault, false),
                compat::AccountMeta::new(self.magic_context, false),
                compat::AccountMeta::new_readonly(self.magic_program, false),
            ],
            data: vec![EphemeralSplDiscriminator::EnsureTransferQueueCrank as u8],
        }
    }
}

impl Default for EnsureTransferQueueCrankBuilder {
    fn default() -> Self {
        Self {
            payer: compat::Pubkey::default(),
            queue: compat::Pubkey::default(),
            magic_fee_vault: compat::Pubkey::default(),
            magic_context: MAGIC_CONTEXT_ID,
            magic_program: MAGIC_PROGRAM_ID,
        }
    }
}
