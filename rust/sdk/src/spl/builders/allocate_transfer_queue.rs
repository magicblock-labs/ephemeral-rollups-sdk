use crate::{consts::ESPL_TOKEN_PROGRAM_ID, spl::EphemeralSplDiscriminator};

pub struct AllocateTransferQueueBuilder {
    pub queue: solana_program::pubkey::Pubkey,
}

impl AllocateTransferQueueBuilder {
    #[inline(always)]
    pub fn instruction(&self) -> solana_program::instruction::Instruction {
        solana_program::instruction::Instruction {
            program_id: ESPL_TOKEN_PROGRAM_ID,
            accounts: vec![
                solana_program::instruction::AccountMeta::new(self.queue, false),
                solana_program::instruction::AccountMeta::new_readonly(
                    solana_program::pubkey::Pubkey::new_from_array(
                        solana_system_interface::program::ID.to_bytes(),
                    ),
                    false,
                ),
            ],
            data: vec![EphemeralSplDiscriminator::AllocateTransferQueue as u8],
        }
    }
}
