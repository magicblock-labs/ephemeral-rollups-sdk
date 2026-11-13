use crate::{
    consts::ESPL_TOKEN_PROGRAM_ID,
    spl::{EphemeralAta, EphemeralSplDiscriminator},
};

pub struct CloseEphemeralAtaBuilder {
    pub payer: solana_program::pubkey::Pubkey,
    pub user: solana_program::pubkey::Pubkey,
    pub mint: solana_program::pubkey::Pubkey,
}

impl CloseEphemeralAtaBuilder {
    #[inline(always)]
    pub fn instruction(&self) -> solana_program::instruction::Instruction {
        let (eata, _eata_bump) = EphemeralAta::find_pda(&self.user, &self.mint);
        solana_program::instruction::Instruction {
            program_id: ESPL_TOKEN_PROGRAM_ID,
            accounts: vec![
                solana_program::instruction::AccountMeta::new_readonly(self.user, true),
                solana_program::instruction::AccountMeta::new(eata, false),
                solana_program::instruction::AccountMeta::new(self.payer, false),
            ],
            data: vec![EphemeralSplDiscriminator::CloseEphemeralAta as u8],
        }
    }
}
