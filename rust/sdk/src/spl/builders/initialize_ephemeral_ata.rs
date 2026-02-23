use crate::{
    consts::ESPL_TOKEN_PROGRAM_ID,
    solana_compat::solana::{system_program, AccountMeta, Instruction, Pubkey},
    spl::{cpi::EphemeralSplDiscriminator, EphemeralAta},
};

pub struct InitializeEphemeralAtaBuilder {
    pub payer: Pubkey,
    pub user: Pubkey,
    pub mint: Pubkey,
}

impl InitializeEphemeralAtaBuilder {
    #[inline(always)]
    pub fn instruction(&self) -> Instruction {
        let (eata, eata_bump) = EphemeralAta::find_pda(&self.user, &self.mint);
        Instruction {
            program_id: ESPL_TOKEN_PROGRAM_ID,
            accounts: vec![
                AccountMeta::new(eata, false),
                AccountMeta::new(self.payer, true),
                AccountMeta::new_readonly(self.user, false),
                AccountMeta::new_readonly(self.mint, false),
                AccountMeta::new_readonly(system_program::id(), false),
            ],
            data: vec![
                EphemeralSplDiscriminator::InitializeEphemeralAta as u8,
                eata_bump,
            ],
        }
    }
}
