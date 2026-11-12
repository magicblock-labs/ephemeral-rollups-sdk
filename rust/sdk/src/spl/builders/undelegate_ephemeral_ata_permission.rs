use crate::{
    access_control::structs::Permission,
    consts::{ESPL_TOKEN_PROGRAM_ID, MAGIC_CONTEXT_ID, MAGIC_PROGRAM_ID, PERMISSION_PROGRAM_ID},
    spl::{EphemeralAta, EphemeralSplDiscriminator},
};

pub struct UndelegateEphemeralAtaPermissionBuilder {
    pub payer: solana_program::pubkey::Pubkey,
    pub user: solana_program::pubkey::Pubkey,
    pub mint: solana_program::pubkey::Pubkey,
}

impl UndelegateEphemeralAtaPermissionBuilder {
    #[inline(always)]
    pub fn instruction(&self) -> solana_program::instruction::Instruction {
        let (eata, _eata_bump) = EphemeralAta::find_pda(&self.user, &self.mint);
        let (permission, _permission_bump) = Permission::find_pda(&eata);

        solana_program::instruction::Instruction {
            program_id: ESPL_TOKEN_PROGRAM_ID,
            accounts: vec![
                solana_program::instruction::AccountMeta::new_readonly(self.payer, true),
                solana_program::instruction::AccountMeta::new(eata, false),
                solana_program::instruction::AccountMeta::new(permission, false),
                solana_program::instruction::AccountMeta::new_readonly(
                    PERMISSION_PROGRAM_ID,
                    false,
                ),
                solana_program::instruction::AccountMeta::new_readonly(MAGIC_PROGRAM_ID, false),
                solana_program::instruction::AccountMeta::new(MAGIC_CONTEXT_ID, false),
            ],
            data: vec![EphemeralSplDiscriminator::UndelegateEphemeralAtaPermission as u8],
        }
    }
}
