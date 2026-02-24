use crate::{
    consts::ESPL_TOKEN_PROGRAM_ID,
    solana_compat::solana::{AccountMeta, Instruction, Pubkey},
    spl::{EphemeralAta, EphemeralSplDiscriminator},
};

pub struct CloseEphemeralAtaBuilder {
    pub payer: Pubkey,
    pub user: Pubkey,
    pub mint: Pubkey,
}

impl CloseEphemeralAtaBuilder {
    #[inline(always)]
    pub fn instruction(&self) -> Instruction {
        let (eata, _eata_bump) = EphemeralAta::find_pda(&self.user, &self.mint);
        Instruction {
            program_id: ESPL_TOKEN_PROGRAM_ID,
            accounts: vec![
                AccountMeta::new_readonly(self.user, true),
                AccountMeta::new(eata, false),
                AccountMeta::new(self.payer, false),
            ],
            data: vec![EphemeralSplDiscriminator::CloseEphemeralAta as u8],
        }
    }
}
