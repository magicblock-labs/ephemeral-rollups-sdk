use crate::{
    consts::ESPL_TOKEN_PROGRAM_ID,
    solana_compat::solana::{
        invoke, invoke_signed, AccountInfo, AccountMeta, Instruction, ProgramResult,
    },
    spl::EphemeralSplDiscriminator,
};

/// Deposit SPL tokens into an ephemeral ATA.
pub struct DepositSplTokens<'a> {
    pub authority: AccountInfo<'a>,
    pub eata: AccountInfo<'a>,
    pub vault: AccountInfo<'a>,
    pub mint: AccountInfo<'a>,
    pub user_source_token_acc: AccountInfo<'a>,
    pub vault_token_acc: AccountInfo<'a>,
    pub token_program: AccountInfo<'a>,
    pub amount: u64,
}

impl<'a> DepositSplTokens<'a> {
    #[inline(always)]
    pub fn instruction(&self) -> Instruction {
        let mut data = Vec::with_capacity(9);
        data.push(EphemeralSplDiscriminator::DepositSplTokens as u8);
        data.extend_from_slice(self.amount.to_le_bytes().as_ref());
        Instruction {
            program_id: ESPL_TOKEN_PROGRAM_ID,
            accounts: vec![
                AccountMeta::new(*self.eata.key, false),
                AccountMeta::new_readonly(*self.vault.key, false),
                AccountMeta::new_readonly(*self.mint.key, false),
                AccountMeta::new(*self.user_source_token_acc.key, false),
                AccountMeta::new(*self.vault_token_acc.key, false),
                AccountMeta::new_readonly(*self.authority.key, true),
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
                self.user_source_token_acc.clone(),
                self.vault_token_acc.clone(),
                self.authority.clone(),
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
                self.user_source_token_acc.clone(),
                self.vault_token_acc.clone(),
                self.authority.clone(),
                self.token_program.clone(),
            ],
            signers_seeds,
        )
    }
}
