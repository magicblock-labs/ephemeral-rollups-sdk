use crate::{
    access_control::structs::Permission,
    compat,
    consts::{ESPL_TOKEN_PROGRAM_ID, PERMISSION_PROGRAM_ID},
    spl::{EphemeralAta, EphemeralSplDiscriminator},
};

/// For details on the flag byte, see the [MemberFlags](`crate::access_control::structs::Member`) struct.
pub struct ResetEphemeralAtaPermissionBuilder {
    pub user: compat::Pubkey,
    pub mint: compat::Pubkey,
    pub flag_byte: u8,
}

impl ResetEphemeralAtaPermissionBuilder {
    #[inline(always)]
    pub fn instruction(&self) -> compat::Instruction {
        let (eata, _eata_bump) = EphemeralAta::find_pda(&self.user, &self.mint);
        let (permission, _permission_bump) = Permission::find_pda(&eata);

        compat::Instruction {
            program_id: ESPL_TOKEN_PROGRAM_ID,
            accounts: vec![
                compat::AccountMeta::new_readonly(eata, false),
                compat::AccountMeta::new(permission, false),
                compat::AccountMeta::new_readonly(self.user, true),
                compat::AccountMeta::new_readonly(PERMISSION_PROGRAM_ID, false),
            ],
            data: vec![
                EphemeralSplDiscriminator::ResetEphemeralAtaPermission as u8,
                self.flag_byte,
            ],
        }
    }
}
