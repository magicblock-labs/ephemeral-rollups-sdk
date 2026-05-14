use crate::pda::{
    delegate_buffer_pda_from_delegated_account_and_owner_program,
    delegation_metadata_pda_from_delegated_account, delegation_record_pda_from_delegated_account,
};

use crate::{
    compat,
    consts::ESPL_TOKEN_PROGRAM_ID,
    cpi::DELEGATION_PROGRAM_ID,
    spl::{find_shuttle_ata, find_shuttle_ephemeral_ata, EphemeralSplDiscriminator},
};

pub struct DelegateShuttleEphemeralAtaBuilder {
    pub payer: compat::Pubkey,
    pub owner: compat::Pubkey,
    pub mint: compat::Pubkey,
    pub shuttle_id: u32,
    pub validator: Option<compat::Pubkey>,
}

impl DelegateShuttleEphemeralAtaBuilder {
    #[inline(always)]
    pub fn instruction(&self) -> compat::Instruction {
        let (shuttle_ephemeral_ata, _shuttle_bump) =
            find_shuttle_ephemeral_ata(&self.owner, &self.mint, self.shuttle_id);
        let (shuttle_ata, _shuttle_ata_bump) = find_shuttle_ata(&shuttle_ephemeral_ata, &self.mint);
        let delegation_buffer = delegate_buffer_pda_from_delegated_account_and_owner_program(
            &shuttle_ata,
            &ESPL_TOKEN_PROGRAM_ID,
        );
        let delegation_record = delegation_record_pda_from_delegated_account(&shuttle_ata);
        let delegation_metadata = delegation_metadata_pda_from_delegated_account(&shuttle_ata);

        let mut data = Vec::with_capacity(if self.validator.is_some() { 33 } else { 1 });
        data.push(EphemeralSplDiscriminator::DelegateShuttleEphemeralAta as u8);
        if let Some(validator) = self.validator {
            data.extend_from_slice(validator.as_ref());
        }

        compat::Instruction {
            program_id: ESPL_TOKEN_PROGRAM_ID,
            accounts: vec![
                compat::AccountMeta::new(self.payer, true),
                compat::AccountMeta::new_readonly(shuttle_ephemeral_ata, false),
                compat::AccountMeta::new(shuttle_ata, false),
                compat::AccountMeta::new_readonly(ESPL_TOKEN_PROGRAM_ID, false),
                compat::AccountMeta::new(delegation_buffer, false),
                compat::AccountMeta::new(delegation_record, false),
                compat::AccountMeta::new(delegation_metadata, false),
                compat::AccountMeta::new_readonly(DELEGATION_PROGRAM_ID, false),
                compat::AccountMeta::new_readonly(
                    compat::Pubkey::new_from_array(solana_system_interface::program::ID.to_bytes()),
                    false,
                ),
            ],
            data,
        }
    }
}
