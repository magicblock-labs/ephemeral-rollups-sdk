use crate::{
    consts::{ESPL_TOKEN_PROGRAM_ID, PERMISSION_PROGRAM_ID},
    solana_compat::solana::{system_program, AccountMeta, Instruction, Pubkey},
    spl::{find_transfer_queue, EphemeralSplDiscriminator},
};

const PERMISSION_SEED: &[u8] = b"permission:";

pub struct InitializeTransferQueueBuilder {
    pub payer: Pubkey,
    pub mint: Pubkey,
    pub validator: Pubkey,
    pub requested_items: Option<u32>,
}

impl InitializeTransferQueueBuilder {
    #[inline(always)]
    pub fn instruction(&self) -> Instruction {
        let (queue, _queue_bump) = find_transfer_queue(&self.mint, &self.validator);
        let (queue_permission, _permission_bump) = Pubkey::find_program_address(
            &[PERMISSION_SEED, queue.as_ref()],
            &PERMISSION_PROGRAM_ID,
        );

        let mut data = Vec::with_capacity(5);
        data.push(EphemeralSplDiscriminator::InitializeTransferQueue as u8);
        if let Some(requested_items) = self.requested_items {
            data.extend_from_slice(&requested_items.to_le_bytes());
        }

        Instruction {
            program_id: ESPL_TOKEN_PROGRAM_ID,
            accounts: vec![
                AccountMeta::new(self.payer, true),
                AccountMeta::new(queue, false),
                AccountMeta::new(queue_permission, false),
                AccountMeta::new_readonly(self.mint, false),
                AccountMeta::new_readonly(self.validator, false),
                AccountMeta::new_readonly(system_program::id(), false),
                AccountMeta::new_readonly(PERMISSION_PROGRAM_ID, false),
            ],
            data,
        }
    }
}
