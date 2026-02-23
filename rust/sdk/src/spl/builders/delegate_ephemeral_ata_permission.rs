use dlp::pda::{
    delegate_buffer_pda_from_delegated_account_and_owner_program,
    delegation_metadata_pda_from_delegated_account, delegation_record_pda_from_delegated_account,
};

use crate::{
    access_control::structs::Permission,
    consts::{ESPL_TOKEN_PROGRAM_ID, PERMISSION_PROGRAM_ID},
    cpi::DELEGATION_PROGRAM_ID,
    solana_compat::solana::{system_program, AccountMeta, Instruction, Pubkey},
    spl::{cpi::EphemeralSplDiscriminator, EphemeralAta},
};

pub struct DelegateEphemeralAtaPermissionBuilder {
    pub payer: Pubkey,
    pub user: Pubkey,
    pub mint: Pubkey,
    pub validator: Pubkey,
}

impl DelegateEphemeralAtaPermissionBuilder {
    #[inline(always)]
    pub fn instruction(&self) -> Instruction {
        let (eata, eata_bump) = EphemeralAta::find_pda(&self.user, &self.mint);
        let (permission, _permission_bump) = Permission::find_pda(&eata);
        let delegation_buffer = delegate_buffer_pda_from_delegated_account_and_owner_program(
            &permission,
            &PERMISSION_PROGRAM_ID,
        );
        let delegation_record = delegation_record_pda_from_delegated_account(&permission);
        let delegation_metadata = delegation_metadata_pda_from_delegated_account(&permission);

        Instruction {
            program_id: ESPL_TOKEN_PROGRAM_ID,
            accounts: vec![
                AccountMeta::new(self.payer, true),
                AccountMeta::new(eata, false),
                AccountMeta::new_readonly(PERMISSION_PROGRAM_ID, false),
                AccountMeta::new(permission, false),
                AccountMeta::new_readonly(system_program::id(), false),
                AccountMeta::new(delegation_buffer, false),
                AccountMeta::new(delegation_record, false),
                AccountMeta::new(delegation_metadata, false),
                AccountMeta::new_readonly(DELEGATION_PROGRAM_ID, false),
                AccountMeta::new_readonly(self.validator, false),
            ],
            data: vec![
                EphemeralSplDiscriminator::DelegateEphemeralAtaPermission as u8,
                eata_bump,
            ],
        }
    }
}
