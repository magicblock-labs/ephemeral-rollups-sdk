use crate::{
    access_control::structs::Permission,
    consts::{ESPL_TOKEN_PROGRAM_ID, PERMISSION_PROGRAM_ID},
    solana_compat::solana::{AccountMeta, Instruction, Pubkey},
    spl::{EphemeralAta, EphemeralSplDiscriminator},
};

/// For details on the flag byte, see the [MemberFlags](`crate::access_control::structs::Member`) struct.
pub struct ResetEphemeralAtaPermissionBuilder {
    pub user: Pubkey,
    pub mint: Pubkey,
    pub flag_byte: u8,
}

impl ResetEphemeralAtaPermissionBuilder {
    #[inline(always)]
    pub fn instruction(&self) -> Instruction {
        let (eata, eata_bump) = EphemeralAta::find_pda(&self.user, &self.mint);
        let (permission, _permission_bump) = Permission::find_pda(&eata);

        Instruction {
            program_id: ESPL_TOKEN_PROGRAM_ID,
            accounts: vec![
                AccountMeta::new(eata, false),
                AccountMeta::new(permission, false),
                AccountMeta::new_readonly(self.user, true),
                AccountMeta::new_readonly(PERMISSION_PROGRAM_ID, false),
            ],
            data: vec![
                EphemeralSplDiscriminator::ResetEphemeralAtaPermission as u8,
                eata_bump,
                self.flag_byte,
            ],
        }
    }
}
