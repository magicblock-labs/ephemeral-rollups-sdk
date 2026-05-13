use crate::{compat, consts::ESPL_TOKEN_PROGRAM_ID, spl::EphemeralSplDiscriminator};

pub struct AllocateTransferQueueBuilder {
    pub queue: compat::Pubkey,
}

impl AllocateTransferQueueBuilder {
    #[inline(always)]
    pub fn instruction(&self) -> compat::Instruction {
        compat::Instruction {
            program_id: ESPL_TOKEN_PROGRAM_ID,
            accounts: vec![
                compat::AccountMeta::new(self.queue, false),
                compat::AccountMeta::new_readonly(
                    compat::Pubkey::new_from_array(solana_system_interface::program::ID.to_bytes()),
                    false,
                ),
            ],
            data: vec![EphemeralSplDiscriminator::AllocateTransferQueue as u8],
        }
    }
}
