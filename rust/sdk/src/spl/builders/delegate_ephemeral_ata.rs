use crate::pda::{
    delegate_buffer_pda_from_delegated_account_and_owner_program,
    delegation_metadata_pda_from_delegated_account, delegation_record_pda_from_delegated_account,
};

use crate::{
    consts::ESPL_TOKEN_PROGRAM_ID,
    cpi::DELEGATION_PROGRAM_ID,
    spl::{EphemeralAta, EphemeralSplDiscriminator},
};

pub struct DelegateEphemeralAtaBuilder {
    pub payer: solana_program::pubkey::Pubkey,
    pub user: solana_program::pubkey::Pubkey,
    pub mint: solana_program::pubkey::Pubkey,
    pub validator: Option<solana_program::pubkey::Pubkey>,
}

impl DelegateEphemeralAtaBuilder {
    #[inline(always)]
    pub fn instruction(&self) -> solana_program::instruction::Instruction {
        let (eata, _eata_bump) = EphemeralAta::find_pda(&self.user, &self.mint);
        let delegation_buffer = delegate_buffer_pda_from_delegated_account_and_owner_program(
            &eata,
            &ESPL_TOKEN_PROGRAM_ID,
        );
        let delegation_record = delegation_record_pda_from_delegated_account(&eata);
        let delegation_metadata = delegation_metadata_pda_from_delegated_account(&eata);

        let mut data = Vec::with_capacity(33);
        data.push(EphemeralSplDiscriminator::DelegateEphemeralAta as u8);
        if let Some(validator) = self.validator {
            data.extend_from_slice(validator.as_ref());
        }

        solana_program::instruction::Instruction {
            program_id: ESPL_TOKEN_PROGRAM_ID,
            accounts: vec![
                solana_program::instruction::AccountMeta::new(self.payer, true),
                solana_program::instruction::AccountMeta::new(eata, false),
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
                    solana_program::pubkey::Pubkey::new_from_array(
                        solana_system_interface::program::ID.to_bytes(),
                    ),
                    false,
                ),
            ],
            data,
        }
    }
}
