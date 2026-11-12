use crate::{
    consts::{ESPL_TOKEN_PROGRAM_ID, MAGIC_CONTEXT_ID, MAGIC_PROGRAM_ID, TOKEN_PROGRAM_ID},
    spl::{find_associated_token_address_with_bump, EphemeralAta, EphemeralSplDiscriminator},
};

pub struct UndelegateEphemeralAtaBuilder {
    pub payer: solana_program::pubkey::Pubkey,
    pub user: solana_program::pubkey::Pubkey,
    pub mint: solana_program::pubkey::Pubkey,
}

impl UndelegateEphemeralAtaBuilder {
    #[inline(always)]
    pub fn instruction(&self) -> solana_program::instruction::Instruction {
        let (eata, _eata_bump) = EphemeralAta::find_pda(&self.user, &self.mint);
        let user_ata =
            find_associated_token_address_with_bump(&self.user, &self.mint, &TOKEN_PROGRAM_ID).0;

        solana_program::instruction::Instruction {
            program_id: ESPL_TOKEN_PROGRAM_ID,
            accounts: vec![
                solana_program::instruction::AccountMeta::new_readonly(self.payer, true),
                solana_program::instruction::AccountMeta::new(user_ata, false),
                solana_program::instruction::AccountMeta::new_readonly(eata, false),
                solana_program::instruction::AccountMeta::new(MAGIC_CONTEXT_ID, false),
                solana_program::instruction::AccountMeta::new_readonly(MAGIC_PROGRAM_ID, false),
            ],
            data: vec![EphemeralSplDiscriminator::UndelegateEphemeralAta as u8],
        }
    }
}
