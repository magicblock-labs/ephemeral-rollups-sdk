use crate::{
    access_control::structs::Permission,
    consts::{ESPL_TOKEN_PROGRAM_ID, PERMISSION_PROGRAM_ID},
    spl::{EphemeralAta, EphemeralSplDiscriminator},
};

/// For details on the flag byte, see the [MemberFlags](`crate::access_control::structs::Member`) struct.
pub struct ResetEphemeralAtaPermissionBuilder {
    pub user: solana_program::pubkey::Pubkey,
    pub mint: solana_program::pubkey::Pubkey,
    pub flag_byte: u8,
}

impl ResetEphemeralAtaPermissionBuilder {
    #[inline(always)]
    pub fn instruction(&self) -> solana_program::instruction::Instruction {
        let (eata, _eata_bump) = EphemeralAta::find_pda(&self.user, &self.mint);
        let (permission, _permission_bump) = Permission::find_pda(&eata);

        solana_program::instruction::Instruction {
            program_id: ESPL_TOKEN_PROGRAM_ID,
            accounts: vec![
                solana_program::instruction::AccountMeta::new_readonly(eata, false),
                solana_program::instruction::AccountMeta::new(permission, false),
                solana_program::instruction::AccountMeta::new_readonly(self.user, true),
                solana_program::instruction::AccountMeta::new_readonly(
                    PERMISSION_PROGRAM_ID,
                    false,
                ),
            ],
            data: vec![
                EphemeralSplDiscriminator::ResetEphemeralAtaPermission as u8,
                self.flag_byte,
            ],
        }
    }
}
