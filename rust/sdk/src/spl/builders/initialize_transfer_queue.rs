use crate::{
    compat,
    consts::{ESPL_TOKEN_PROGRAM_ID, PERMISSION_PROGRAM_ID},
    spl::{find_transfer_queue, EphemeralSplDiscriminator},
};

const PERMISSION_SEED: &[u8] = b"permission:";

pub struct InitializeTransferQueueBuilder {
    pub payer: compat::Pubkey,
    pub mint: compat::Pubkey,
    pub validator: compat::Pubkey,
    pub requested_items: Option<u32>,
}

impl InitializeTransferQueueBuilder {
    #[inline(always)]
    pub fn instruction(&self) -> compat::Instruction {
        let (queue, _queue_bump) = find_transfer_queue(&self.mint, &self.validator);
        let (queue_permission, _permission_bump) = compat::Pubkey::find_program_address(
            &[PERMISSION_SEED, queue.as_ref()],
            &PERMISSION_PROGRAM_ID,
        );

        let mut data = Vec::with_capacity(5);
        data.push(EphemeralSplDiscriminator::InitializeTransferQueue as u8);
        if let Some(requested_items) = self.requested_items {
            data.extend_from_slice(&requested_items.to_le_bytes());
        }

        compat::Instruction {
            program_id: ESPL_TOKEN_PROGRAM_ID,
            accounts: vec![
                compat::AccountMeta::new(self.payer, true),
                compat::AccountMeta::new(queue, false),
                compat::AccountMeta::new(queue_permission, false),
                compat::AccountMeta::new_readonly(self.mint, false),
                compat::AccountMeta::new_readonly(self.validator, false),
                compat::AccountMeta::new_readonly(
                    compat::Pubkey::new_from_array(solana_system_interface::program::ID.to_bytes()),
                    false,
                ),
                compat::AccountMeta::new_readonly(PERMISSION_PROGRAM_ID, false),
            ],
            data,
        }
    }
}
