use crate::{
    compat,
    consts::{ESPL_TOKEN_PROGRAM_ID, MAGIC_CONTEXT_ID, MAGIC_PROGRAM_ID, TOKEN_PROGRAM_ID},
    spl::{find_associated_token_address_with_bump, EphemeralAta, EphemeralSplDiscriminator},
};

pub struct UndelegateEphemeralAtaBuilder {
    pub payer: compat::Pubkey,
    pub user: compat::Pubkey,
    pub mint: compat::Pubkey,
}

impl UndelegateEphemeralAtaBuilder {
    #[inline(always)]
    pub fn instruction(&self) -> compat::Instruction {
        let (eata, _eata_bump) = EphemeralAta::find_pda(&self.user, &self.mint);
        let user_ata =
            find_associated_token_address_with_bump(&self.user, &self.mint, &TOKEN_PROGRAM_ID).0;

        compat::Instruction {
            program_id: ESPL_TOKEN_PROGRAM_ID,
            accounts: vec![
                compat::AccountMeta::new_readonly(self.payer, true),
                compat::AccountMeta::new(user_ata, false),
                compat::AccountMeta::new_readonly(eata, false),
                compat::AccountMeta::new(MAGIC_CONTEXT_ID, false),
                compat::AccountMeta::new_readonly(MAGIC_PROGRAM_ID, false),
            ],
            data: vec![EphemeralSplDiscriminator::UndelegateEphemeralAta as u8],
        }
    }
}
