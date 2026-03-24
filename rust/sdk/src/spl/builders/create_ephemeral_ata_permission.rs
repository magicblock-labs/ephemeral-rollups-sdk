use crate::{
    access_control::structs::Permission,
    consts::{ESPL_TOKEN_PROGRAM_ID, PERMISSION_PROGRAM_ID},
    solana_compat::solana::{system_program, AccountMeta, Instruction, Pubkey},
    spl::{EphemeralAta, EphemeralSplDiscriminator},
};

/// For details on the flag byte, see the [MemberFlags](`crate::access_control::structs::Member`) struct.
pub struct CreateEphemeralAtaPermissionBuilder {
    pub payer: Pubkey,
    pub user: Pubkey,
    pub mint: Pubkey,
    pub flag_byte: u8,
}

impl CreateEphemeralAtaPermissionBuilder {
    #[inline(always)]
    pub fn instruction(&self) -> Instruction {
        let (eata, _eata_bump) = EphemeralAta::find_pda(&self.user, &self.mint);
        let (permission, _permission_bump) = Permission::find_pda(&eata);

        Instruction {
            program_id: ESPL_TOKEN_PROGRAM_ID,
            accounts: vec![
                AccountMeta::new(eata, false),
                AccountMeta::new(permission, false),
                AccountMeta::new(self.payer, true),
                AccountMeta::new_readonly(system_program::id(), false),
                AccountMeta::new_readonly(PERMISSION_PROGRAM_ID, false),
            ],
            data: vec![
                EphemeralSplDiscriminator::CreateEphemeralAtaPermission as u8,
                self.flag_byte,
            ],
        }
    }
}
