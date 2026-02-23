use crate::{
    consts::ESPL_TOKEN_PROGRAM_ID,
    solana_compat::solana::{
        invoke, invoke_signed, AccountInfo, AccountMeta, Instruction, ProgramResult,
    },
    spl::EphemeralSplDiscriminator,
};

/// Build a withdraw SPL tokens instruction.
pub struct WithdrawSplTokens<'a> {
    pub payer: AccountInfo<'a>,
    pub eata: AccountInfo<'a>,
    pub vault: AccountInfo<'a>,
    pub mint: AccountInfo<'a>,
    pub vault_ata: AccountInfo<'a>,
    pub user_ata: AccountInfo<'a>,
    pub token_program: AccountInfo<'a>,
    pub eata_bump: u8,
    pub amount: u64,
}

impl<'a> WithdrawSplTokens<'a> {
    #[inline(always)]
    pub fn instruction(&self) -> Instruction {
        let mut data = Vec::with_capacity(10);
        data.push(EphemeralSplDiscriminator::WithdrawSplTokens as u8);
        data.extend_from_slice(self.amount.to_le_bytes().as_ref());
        data.push(self.eata_bump);
        Instruction {
            program_id: ESPL_TOKEN_PROGRAM_ID,
            accounts: vec![
                AccountMeta::new(*self.eata.key, false),
                AccountMeta::new_readonly(*self.vault.key, false),
                AccountMeta::new_readonly(*self.mint.key, false),
                AccountMeta::new(*self.vault_ata.key, false),
                AccountMeta::new(*self.user_ata.key, false),
                AccountMeta::new_readonly(*self.payer.key, true),
                AccountMeta::new_readonly(*self.token_program.key, false),
            ],
            data,
        }
    }

    #[inline(always)]
    pub fn invoke(&self) -> ProgramResult {
        invoke(
            &self.instruction(),
            &[
                self.eata.clone(),
                self.vault.clone(),
                self.mint.clone(),
                self.vault_ata.clone(),
                self.user_ata.clone(),
                self.payer.clone(),
                self.token_program.clone(),
            ],
        )
    }

    #[inline(always)]
    pub fn invoke_signed(&self, signers_seeds: &[&[&[u8]]]) -> ProgramResult {
        invoke_signed(
            &self.instruction(),
            &[
                self.eata.clone(),
                self.vault.clone(),
                self.mint.clone(),
                self.vault_ata.clone(),
                self.user_ata.clone(),
                self.payer.clone(),
                self.token_program.clone(),
            ],
            signers_seeds,
        )
    }
}
