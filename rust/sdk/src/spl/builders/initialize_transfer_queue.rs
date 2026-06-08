use crate::{
    compat,
    consts::{
        ASSOCIATED_TOKEN_PROGRAM_ID, ESPL_TOKEN_PROGRAM_ID, PERMISSION_PROGRAM_ID, TOKEN_PROGRAM_ID,
    },
    cpi::DELEGATION_PROGRAM_ID,
    pda::{
        delegate_buffer_pda_from_delegated_account_and_owner_program,
        delegation_metadata_pda_from_delegated_account,
        delegation_record_pda_from_delegated_account,
    },
    spl::{
        find_transfer_queue, find_transfer_queue_ephemeral_ata, find_transfer_queue_vault_ata,
        EphemeralSplDiscriminator,
    },
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
        let (queue_ephemeral_ata, _queue_eata_bump) =
            find_transfer_queue_ephemeral_ata(&self.mint, &self.validator);
        let queue_vault_ata =
            find_transfer_queue_vault_ata(&self.mint, &self.validator, &TOKEN_PROGRAM_ID);
        let delegation_buffer = delegate_buffer_pda_from_delegated_account_and_owner_program(
            &queue_ephemeral_ata,
            &ESPL_TOKEN_PROGRAM_ID,
        );
        let delegation_record = delegation_record_pda_from_delegated_account(&queue_ephemeral_ata);
        let delegation_metadata =
            delegation_metadata_pda_from_delegated_account(&queue_ephemeral_ata);

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
                compat::AccountMeta::new(queue_ephemeral_ata, false),
                compat::AccountMeta::new(queue_vault_ata, false),
                compat::AccountMeta::new_readonly(TOKEN_PROGRAM_ID, false),
                compat::AccountMeta::new_readonly(ASSOCIATED_TOKEN_PROGRAM_ID, false),
                compat::AccountMeta::new_readonly(ESPL_TOKEN_PROGRAM_ID, false),
                compat::AccountMeta::new(delegation_buffer, false),
                compat::AccountMeta::new(delegation_record, false),
                compat::AccountMeta::new(delegation_metadata, false),
                compat::AccountMeta::new_readonly(DELEGATION_PROGRAM_ID, false),
            ],
            data,
        }
    }
}
