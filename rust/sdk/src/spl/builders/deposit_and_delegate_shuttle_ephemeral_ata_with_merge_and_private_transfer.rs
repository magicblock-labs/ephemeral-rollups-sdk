use dlp_api::pda::{
    delegate_buffer_pda_from_delegated_account_and_owner_program,
    delegation_metadata_pda_from_delegated_account, delegation_record_pda_from_delegated_account,
};

use crate::{
    consts::{ASSOCIATED_TOKEN_PROGRAM_ID, ESPL_TOKEN_PROGRAM_ID, TOKEN_PROGRAM_ID},
    cpi::DELEGATION_PROGRAM_ID,
    solana_compat::solana::{system_program, AccountMeta, Instruction, Pubkey},
    spl::{
        find_rent_pda, find_shuttle_ata, find_shuttle_ephemeral_ata, find_shuttle_wallet_ata,
        find_transfer_queue, find_vault_ata, EphemeralSplDiscriminator, GlobalVault,
    },
};

pub struct DepositAndDelegateShuttleEphemeralAtaWithMergeAndPrivateTransferBuilder {
    pub payer: Pubkey,
    pub owner: Pubkey,
    pub mint: Pubkey,
    pub source_ata: Pubkey,
    pub destination_ata: Pubkey,
    pub shuttle_id: u32,
    pub amount: u64,
    pub min_delay_ms: u64,
    pub max_delay_ms: u64,
    pub split: u32,
    pub validator: Option<Pubkey>,
}

impl DepositAndDelegateShuttleEphemeralAtaWithMergeAndPrivateTransferBuilder {
    #[inline(always)]
    pub fn instruction(&self) -> Instruction {
        let (rent_pda, _rent_bump) = find_rent_pda();
        let (shuttle_ephemeral_ata, _shuttle_bump) =
            find_shuttle_ephemeral_ata(&self.owner, &self.mint, self.shuttle_id);
        let (shuttle_ata, _shuttle_ata_bump) = find_shuttle_ata(&shuttle_ephemeral_ata, &self.mint);
        let shuttle_wallet_ata = find_shuttle_wallet_ata(&self.mint, &shuttle_ephemeral_ata);
        let delegation_buffer = delegate_buffer_pda_from_delegated_account_and_owner_program(
            &shuttle_ata,
            &ESPL_TOKEN_PROGRAM_ID,
        );
        let delegation_record = delegation_record_pda_from_delegated_account(&shuttle_ata);
        let delegation_metadata = delegation_metadata_pda_from_delegated_account(&shuttle_ata);
        let (vault, _vault_bump) = GlobalVault::find_pda(&self.mint);
        let vault_ata = find_vault_ata(&self.mint, &vault);
        let (queue, _queue_bump) = find_transfer_queue(&self.mint);

        let mut data = Vec::with_capacity(if self.validator.is_some() { 65 } else { 33 });
        data.push(
            EphemeralSplDiscriminator::DepositAndDelegateShuttleEphemeralAtaWithMergeAndPrivateTransfer
                as u8,
        );
        data.extend_from_slice(&self.shuttle_id.to_le_bytes());
        data.extend_from_slice(&self.amount.to_le_bytes());
        data.extend_from_slice(&self.min_delay_ms.to_le_bytes());
        data.extend_from_slice(&self.max_delay_ms.to_le_bytes());
        data.extend_from_slice(&self.split.to_le_bytes());
        if let Some(validator) = self.validator {
            data.extend_from_slice(validator.as_ref());
        }

        Instruction {
            program_id: ESPL_TOKEN_PROGRAM_ID,
            accounts: vec![
                AccountMeta::new(self.payer, true),
                AccountMeta::new(rent_pda, false),
                AccountMeta::new(shuttle_ephemeral_ata, false),
                AccountMeta::new(shuttle_ata, false),
                AccountMeta::new(shuttle_wallet_ata, false),
                AccountMeta::new_readonly(self.owner, true),
                AccountMeta::new_readonly(ESPL_TOKEN_PROGRAM_ID, false),
                AccountMeta::new(delegation_buffer, false),
                AccountMeta::new(delegation_record, false),
                AccountMeta::new(delegation_metadata, false),
                AccountMeta::new_readonly(DELEGATION_PROGRAM_ID, false),
                AccountMeta::new_readonly(ASSOCIATED_TOKEN_PROGRAM_ID, false),
                AccountMeta::new_readonly(system_program::id(), false),
                AccountMeta::new(self.destination_ata, false),
                AccountMeta::new_readonly(self.mint, false),
                AccountMeta::new_readonly(TOKEN_PROGRAM_ID, false),
                AccountMeta::new_readonly(vault, false),
                AccountMeta::new(self.source_ata, false),
                AccountMeta::new(vault_ata, false),
                AccountMeta::new(queue, false),
            ],
            data,
        }
    }
}
