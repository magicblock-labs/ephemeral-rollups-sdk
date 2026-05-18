use crate::{
    access_control::structs::Permission,
    compat,
    consts::{ESPL_TOKEN_PROGRAM_ID, PERMISSION_PROGRAM_ID},
    spl::{EphemeralAta, EphemeralSplDiscriminator},
};

/// For details on the flag byte, see the [MemberFlags](`crate::access_control::structs::Member`) struct.
pub struct CreateEphemeralAtaPermissionBuilder {
    pub payer: compat::Pubkey,
    pub user: compat::Pubkey,
    pub mint: compat::Pubkey,
    pub flag_byte: u8,
}

impl CreateEphemeralAtaPermissionBuilder {
    #[inline(always)]
    pub fn instruction(&self) -> compat::Instruction {
        let (eata, _eata_bump) = EphemeralAta::find_pda(&self.user, &self.mint);
        let (permission, _permission_bump) = Permission::find_pda(&eata);

        compat::Instruction {
            program_id: ESPL_TOKEN_PROGRAM_ID,
            accounts: vec![
                compat::AccountMeta::new(eata, false),
                compat::AccountMeta::new(permission, false),
                compat::AccountMeta::new(self.payer, true),
                compat::AccountMeta::new_readonly(
                    compat::Pubkey::new_from_array(solana_system_interface::program::ID.to_bytes()),
                    false,
                ),
                compat::AccountMeta::new_readonly(PERMISSION_PROGRAM_ID, false),
            ],
            data: vec![
                EphemeralSplDiscriminator::CreateEphemeralAtaPermission as u8,
                self.flag_byte,
            ],
        }
    }
}
