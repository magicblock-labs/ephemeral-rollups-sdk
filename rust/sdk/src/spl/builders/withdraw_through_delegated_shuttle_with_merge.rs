use crate::pda::{
    delegate_buffer_pda_from_delegated_account_and_owner_program,
    delegation_metadata_pda_from_delegated_account, delegation_record_pda_from_delegated_account,
};

use crate::{
    consts::{ASSOCIATED_TOKEN_PROGRAM_ID, ESPL_TOKEN_PROGRAM_ID, TOKEN_PROGRAM_ID},
    cpi::DELEGATION_PROGRAM_ID,
    spl::{
        find_rent_pda, find_shuttle_ata, find_shuttle_ephemeral_ata, find_shuttle_wallet_ata,
        EphemeralSplDiscriminator,
    },
};

pub struct WithdrawThroughDelegatedShuttleWithMergeBuilder {
    pub payer: solana_program::pubkey::Pubkey,
    pub owner: solana_program::pubkey::Pubkey,
    pub owner_ata: solana_program::pubkey::Pubkey,
    pub mint: solana_program::pubkey::Pubkey,
    pub shuttle_id: u32,
    pub amount: u64,
    pub validator: Option<solana_program::pubkey::Pubkey>,
}

impl WithdrawThroughDelegatedShuttleWithMergeBuilder {
    #[inline(always)]
    pub fn instruction(&self) -> solana_program::instruction::Instruction {
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

        let mut data = Vec::with_capacity(if self.validator.is_some() { 45 } else { 13 });
        data.push(EphemeralSplDiscriminator::WithdrawThroughDelegatedShuttleWithMerge as u8);
        data.extend_from_slice(&self.shuttle_id.to_le_bytes());
        data.extend_from_slice(&self.amount.to_le_bytes());
        if let Some(validator) = self.validator {
            data.extend_from_slice(validator.as_ref());
        }

        solana_program::instruction::Instruction {
            program_id: ESPL_TOKEN_PROGRAM_ID,
            accounts: vec![
                solana_program::instruction::AccountMeta::new(self.payer, true),
                solana_program::instruction::AccountMeta::new(rent_pda, false),
                solana_program::instruction::AccountMeta::new(shuttle_ephemeral_ata, false),
                solana_program::instruction::AccountMeta::new(shuttle_ata, false),
                solana_program::instruction::AccountMeta::new(shuttle_wallet_ata, false),
                solana_program::instruction::AccountMeta::new_readonly(self.owner, true),
                solana_program::instruction::AccountMeta::new_readonly(
                    ESPL_TOKEN_PROGRAM_ID,
                    false,
                ),
                solana_program::instruction::AccountMeta::new(delegation_buffer, false),
                solana_program::instruction::AccountMeta::new(delegation_record, false),
                solana_program::instruction::AccountMeta::new(delegation_metadata, false),
                solana_program::instruction::AccountMeta::new_readonly(
                    DELEGATION_PROGRAM_ID,
                    false,
                ),
                solana_program::instruction::AccountMeta::new_readonly(
                    ASSOCIATED_TOKEN_PROGRAM_ID,
                    false,
                ),
                solana_program::instruction::AccountMeta::new_readonly(
                    solana_program::pubkey::Pubkey::new_from_array(
                        solana_system_interface::program::ID.to_bytes(),
                    ),
                    false,
                ),
                solana_program::instruction::AccountMeta::new(self.owner_ata, false),
                solana_program::instruction::AccountMeta::new_readonly(self.mint, false),
                solana_program::instruction::AccountMeta::new_readonly(TOKEN_PROGRAM_ID, false),
            ],
            data,
        }
    }
}
