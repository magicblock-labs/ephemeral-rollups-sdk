use crate::pda::{
    delegate_buffer_pda_from_delegated_account_and_owner_program,
    delegation_metadata_pda_from_delegated_account, delegation_record_pda_from_delegated_account,
};

use crate::{
    access_control::structs::Permission,
    compat,
    consts::{ESPL_TOKEN_PROGRAM_ID, PERMISSION_PROGRAM_ID},
    cpi::DELEGATION_PROGRAM_ID,
    spl::{EphemeralAta, EphemeralSplDiscriminator},
};

pub struct DelegateEphemeralAtaPermissionBuilder {
    pub payer: compat::Pubkey,
    pub user: compat::Pubkey,
    pub mint: compat::Pubkey,
    pub validator: compat::Pubkey,
}

impl DelegateEphemeralAtaPermissionBuilder {
    #[inline(always)]
    pub fn instruction(&self) -> compat::Instruction {
        let (eata, _eata_bump) = EphemeralAta::find_pda(&self.user, &self.mint);
        let (permission, _permission_bump) = Permission::find_pda(&eata);
        let delegation_buffer = delegate_buffer_pda_from_delegated_account_and_owner_program(
            &permission,
            &PERMISSION_PROGRAM_ID,
        );
        let delegation_record = delegation_record_pda_from_delegated_account(&permission);
        let delegation_metadata = delegation_metadata_pda_from_delegated_account(&permission);

        compat::Instruction {
            program_id: ESPL_TOKEN_PROGRAM_ID,
            accounts: vec![
                compat::AccountMeta::new(self.payer, true),
                compat::AccountMeta::new(eata, false),
                compat::AccountMeta::new_readonly(PERMISSION_PROGRAM_ID, false),
                compat::AccountMeta::new(permission, false),
                compat::AccountMeta::new_readonly(
                    compat::Pubkey::new_from_array(solana_system_interface::program::ID.to_bytes()),
                    false,
                ),
                compat::AccountMeta::new(delegation_buffer, false),
                compat::AccountMeta::new(delegation_record, false),
                compat::AccountMeta::new(delegation_metadata, false),
                compat::AccountMeta::new_readonly(DELEGATION_PROGRAM_ID, false),
                compat::AccountMeta::new_readonly(self.validator, false),
            ],
            data: vec![EphemeralSplDiscriminator::DelegateEphemeralAtaPermission as u8],
        }
    }
}
