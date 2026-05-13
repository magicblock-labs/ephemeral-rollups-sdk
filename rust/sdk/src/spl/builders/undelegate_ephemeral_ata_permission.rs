use crate::{
    access_control::structs::Permission,
    compat,
    consts::{ESPL_TOKEN_PROGRAM_ID, MAGIC_CONTEXT_ID, MAGIC_PROGRAM_ID, PERMISSION_PROGRAM_ID},
    spl::{EphemeralAta, EphemeralSplDiscriminator},
};

pub struct UndelegateEphemeralAtaPermissionBuilder {
    pub payer: compat::Pubkey,
    pub user: compat::Pubkey,
    pub mint: compat::Pubkey,
}

impl UndelegateEphemeralAtaPermissionBuilder {
    #[inline(always)]
    pub fn instruction(&self) -> compat::Instruction {
        let (eata, _eata_bump) = EphemeralAta::find_pda(&self.user, &self.mint);
        let (permission, _permission_bump) = Permission::find_pda(&eata);

        compat::Instruction {
            program_id: ESPL_TOKEN_PROGRAM_ID,
            accounts: vec![
                compat::AccountMeta::new_readonly(self.payer, true),
                compat::AccountMeta::new(eata, false),
                compat::AccountMeta::new(permission, false),
                compat::AccountMeta::new_readonly(PERMISSION_PROGRAM_ID, false),
                compat::AccountMeta::new_readonly(MAGIC_PROGRAM_ID, false),
                compat::AccountMeta::new(MAGIC_CONTEXT_ID, false),
            ],
            data: vec![EphemeralSplDiscriminator::UndelegateEphemeralAtaPermission as u8],
        }
    }
}
