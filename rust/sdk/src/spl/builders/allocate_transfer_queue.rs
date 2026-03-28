use crate::{
    consts::ESPL_TOKEN_PROGRAM_ID,
    solana_compat::solana::{system_program, AccountMeta, Instruction, Pubkey},
    spl::EphemeralSplDiscriminator,
};

pub struct AllocateTransferQueueBuilder {
    pub queue: Pubkey,
}

impl AllocateTransferQueueBuilder {
    #[inline(always)]
    pub fn instruction(&self) -> Instruction {
        Instruction {
            program_id: ESPL_TOKEN_PROGRAM_ID,
            accounts: vec![
                AccountMeta::new(self.queue, false),
                AccountMeta::new_readonly(system_program::id(), false),
            ],
            data: vec![EphemeralSplDiscriminator::AllocateTransferQueue as u8],
        }
    }
}
