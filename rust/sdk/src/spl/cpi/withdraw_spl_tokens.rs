use crate::{
    compat::{self, Compat, Modern},
    consts::ESPL_TOKEN_PROGRAM_ID,
    spl::EphemeralSplDiscriminator,
};
use solana_program::program::{invoke, invoke_signed};

/// Build a withdraw SPL tokens instruction.
pub struct WithdrawSplTokens<'a> {
    pub payer: compat::AccountInfo<'a>,
    pub eata: compat::AccountInfo<'a>,
    pub vault: compat::AccountInfo<'a>,
    pub mint: compat::AccountInfo<'a>,
    pub vault_ata: compat::AccountInfo<'a>,
    pub user_ata: compat::AccountInfo<'a>,
    pub token_program: compat::AccountInfo<'a>,
    pub vault_bump: u8,
    pub amount: u64,
}

impl<'a> WithdrawSplTokens<'a> {
    #[inline(always)]
    pub fn instruction(&self) -> compat::Instruction {
        let mut data = Vec::with_capacity(9);
        data.push(EphemeralSplDiscriminator::WithdrawSplTokens as u8);
        data.extend_from_slice(self.amount.to_le_bytes().as_ref());
        compat::Instruction {
            program_id: ESPL_TOKEN_PROGRAM_ID,
            accounts: vec![
                compat::AccountMeta::new(*self.eata.key, false),
                compat::AccountMeta::new_readonly(*self.vault.key, false),
                compat::AccountMeta::new_readonly(*self.mint.key, false),
                compat::AccountMeta::new(*self.vault_ata.key, false),
                compat::AccountMeta::new(*self.user_ata.key, false),
                compat::AccountMeta::new_readonly(*self.payer.key, true),
                compat::AccountMeta::new_readonly(*self.token_program.key, false),
            ],
            data,
        }
    }

    #[inline(always)]
    pub fn invoke(&self) -> compat::ProgramResult {
        invoke(
            &self.instruction().modern(),
            &[
                self.eata.clone(),
                self.vault.clone(),
                self.mint.clone(),
                self.vault_ata.clone(),
                self.user_ata.clone(),
                self.payer.clone(),
                self.token_program.clone(),
            ]
            .modern(),
        )
        .compat()
    }

    #[inline(always)]
    pub fn invoke_signed(&self, signers_seeds: &[&[&[u8]]]) -> compat::ProgramResult {
        invoke_signed(
            &self.instruction().modern(),
            &[
                self.eata.clone(),
                self.vault.clone(),
                self.mint.clone(),
                self.vault_ata.clone(),
                self.user_ata.clone(),
                self.payer.clone(),
                self.token_program.clone(),
            ]
            .modern(),
            signers_seeds,
        )
        .compat()
    }
}
