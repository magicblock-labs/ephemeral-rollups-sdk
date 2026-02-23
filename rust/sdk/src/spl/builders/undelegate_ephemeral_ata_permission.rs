use crate::{
    access_control::structs::Permission,
    consts::{ESPL_TOKEN_PROGRAM_ID, MAGIC_CONTEXT_ID, MAGIC_PROGRAM_ID, PERMISSION_PROGRAM_ID},
    solana_compat::solana::{AccountMeta, Instruction, Pubkey},
    spl::{cpi::EphemeralSplDiscriminator, EphemeralAta},
};

pub struct UndelegateEphemeralAtaPermissionBuilder {
    pub payer: Pubkey,
    pub user: Pubkey,
    pub mint: Pubkey,
}

impl UndelegateEphemeralAtaPermissionBuilder {
    #[inline(always)]
    pub fn instruction(&self) -> Instruction {
        let (eata, _eata_bump) = EphemeralAta::find_pda(&self.user, &self.mint);
        let (permission, _permission_bump) = Permission::find_pda(&eata);

        Instruction {
            program_id: ESPL_TOKEN_PROGRAM_ID,
            accounts: vec![
                AccountMeta::new_readonly(self.payer, true),
                AccountMeta::new_readonly(eata, false),
                AccountMeta::new(permission, false),
                AccountMeta::new_readonly(PERMISSION_PROGRAM_ID, false),
                AccountMeta::new_readonly(MAGIC_PROGRAM_ID, false),
                AccountMeta::new(MAGIC_CONTEXT_ID, false),
            ],
            data: vec![EphemeralSplDiscriminator::UndelegateEphemeralAtaPermission as u8],
        }
    }
}
