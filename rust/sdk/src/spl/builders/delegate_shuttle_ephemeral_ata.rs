use dlp_api::pda::{
    delegate_buffer_pda_from_delegated_account_and_owner_program,
    delegation_metadata_pda_from_delegated_account, delegation_record_pda_from_delegated_account,
};

use crate::{
    consts::ESPL_TOKEN_PROGRAM_ID,
    cpi::DELEGATION_PROGRAM_ID,
    solana_compat::solana::{system_program, AccountMeta, Instruction, Pubkey},
    spl::{find_shuttle_ata, find_shuttle_ephemeral_ata, EphemeralSplDiscriminator},
};

pub struct DelegateShuttleEphemeralAtaBuilder {
    pub payer: Pubkey,
    pub owner: Pubkey,
    pub mint: Pubkey,
    pub shuttle_id: u32,
    pub validator: Option<Pubkey>,
}

impl DelegateShuttleEphemeralAtaBuilder {
    #[inline(always)]
    pub fn instruction(&self) -> Instruction {
        let (shuttle_ephemeral_ata, _shuttle_bump) =
            find_shuttle_ephemeral_ata(&self.owner, &self.mint, self.shuttle_id);
        let (shuttle_ata, _shuttle_ata_bump) = find_shuttle_ata(&shuttle_ephemeral_ata, &self.mint);
        let delegation_buffer = delegate_buffer_pda_from_delegated_account_and_owner_program(
            &shuttle_ata,
            &ESPL_TOKEN_PROGRAM_ID,
        );
        let delegation_record = delegation_record_pda_from_delegated_account(&shuttle_ata);
        let delegation_metadata = delegation_metadata_pda_from_delegated_account(&shuttle_ata);

        let mut data = Vec::with_capacity(33);
        data.push(EphemeralSplDiscriminator::DelegateShuttleEphemeralAta as u8);
        if let Some(validator) = self.validator {
            data.extend_from_slice(validator.as_ref());
        }

        Instruction {
            program_id: ESPL_TOKEN_PROGRAM_ID,
            accounts: vec![
                AccountMeta::new(self.payer, true),
                AccountMeta::new_readonly(shuttle_ephemeral_ata, false),
                AccountMeta::new(shuttle_ata, false),
                AccountMeta::new_readonly(ESPL_TOKEN_PROGRAM_ID, false),
                AccountMeta::new(delegation_buffer, false),
                AccountMeta::new(delegation_record, false),
                AccountMeta::new(delegation_metadata, false),
                AccountMeta::new_readonly(DELEGATION_PROGRAM_ID, false),
                AccountMeta::new_readonly(system_program::id(), false),
            ],
            data,
        }
    }
}
