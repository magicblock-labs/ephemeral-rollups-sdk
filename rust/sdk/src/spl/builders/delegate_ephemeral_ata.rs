use dlp_api::dlp::pda::{
    delegate_buffer_pda_from_delegated_account_and_owner_program,
    delegation_metadata_pda_from_delegated_account, delegation_record_pda_from_delegated_account,
};

use crate::{
    consts::ESPL_TOKEN_PROGRAM_ID,
    cpi::DELEGATION_PROGRAM_ID,
    solana_compat::solana::{system_program, AccountMeta, Instruction, Pubkey},
    spl::{EphemeralAta, EphemeralSplDiscriminator},
};

pub struct DelegateEphemeralAtaBuilder {
    pub payer: Pubkey,
    pub user: Pubkey,
    pub mint: Pubkey,
    pub validator: Option<Pubkey>,
}

impl DelegateEphemeralAtaBuilder {
    #[inline(always)]
    pub fn instruction(&self) -> Instruction {
        let (eata, eata_bump) = EphemeralAta::find_pda(&self.user, &self.mint);
        let delegation_buffer = delegate_buffer_pda_from_delegated_account_and_owner_program(
            &eata,
            &ESPL_TOKEN_PROGRAM_ID,
        );
        let delegation_record = delegation_record_pda_from_delegated_account(&eata);
        let delegation_metadata = delegation_metadata_pda_from_delegated_account(&eata);

        let mut data = Vec::with_capacity(34);
        data.push(EphemeralSplDiscriminator::DelegateEphemeralAta as u8);
        data.push(eata_bump);
        if let Some(validator) = self.validator {
            data.extend_from_slice(validator.as_ref());
        }

        Instruction {
            program_id: ESPL_TOKEN_PROGRAM_ID,
            accounts: vec![
                AccountMeta::new(self.payer, true),
                AccountMeta::new(eata, false),
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
