use crate::{
    compat,
    consts::ESPL_TOKEN_PROGRAM_ID,
    spl::{EphemeralAta, EphemeralSplDiscriminator},
};

pub struct CloseEphemeralAtaBuilder {
    pub payer: compat::Pubkey,
    pub user: compat::Pubkey,
    pub mint: compat::Pubkey,
}

impl CloseEphemeralAtaBuilder {
    #[inline(always)]
    pub fn instruction(&self) -> compat::Instruction {
        let (eata, _eata_bump) = EphemeralAta::find_pda(&self.user, &self.mint);
        compat::Instruction {
            program_id: ESPL_TOKEN_PROGRAM_ID,
            accounts: vec![
                compat::AccountMeta::new_readonly(self.user, true),
                compat::AccountMeta::new(eata, false),
                compat::AccountMeta::new(self.payer, false),
            ],
            data: vec![EphemeralSplDiscriminator::CloseEphemeralAta as u8],
        }
    }
}
