use crate::{consts::ESPL_TOKEN_PROGRAM_ID, spl::EphemeralSplDiscriminator};
use solana_program::program::{invoke, invoke_signed};

/// Deposit SPL tokens into an ephemeral ATA.
pub struct DepositSplTokens<'a> {
    pub authority: solana_program::account_info::AccountInfo<'a>,
    pub eata: solana_program::account_info::AccountInfo<'a>,
    pub vault: solana_program::account_info::AccountInfo<'a>,
    pub mint: solana_program::account_info::AccountInfo<'a>,
    pub user_source_token_acc: solana_program::account_info::AccountInfo<'a>,
    pub vault_token_acc: solana_program::account_info::AccountInfo<'a>,
    pub token_program: solana_program::account_info::AccountInfo<'a>,
    pub amount: u64,
}

impl<'a> DepositSplTokens<'a> {
    #[inline(always)]
    pub fn instruction(&self) -> solana_program::instruction::Instruction {
        let mut data = Vec::with_capacity(9);
        data.push(EphemeralSplDiscriminator::DepositSplTokens as u8);
        data.extend_from_slice(self.amount.to_le_bytes().as_ref());
        solana_program::instruction::Instruction {
            program_id: ESPL_TOKEN_PROGRAM_ID,
            accounts: vec![
                solana_program::instruction::AccountMeta::new(*self.eata.key, false),
                solana_program::instruction::AccountMeta::new_readonly(*self.vault.key, false),
                solana_program::instruction::AccountMeta::new_readonly(*self.mint.key, false),
                solana_program::instruction::AccountMeta::new(
                    *self.user_source_token_acc.key,
                    false,
                ),
                solana_program::instruction::AccountMeta::new(*self.vault_token_acc.key, false),
                solana_program::instruction::AccountMeta::new_readonly(*self.authority.key, true),
                solana_program::instruction::AccountMeta::new_readonly(
                    *self.token_program.key,
                    false,
                ),
            ],
            data,
        }
    }

    #[inline(always)]
    pub fn invoke(&self) -> solana_program::entrypoint_deprecated::ProgramResult {
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
    pub fn invoke_signed(
        &self,
        signers_seeds: &[&[&[u8]]],
    ) -> solana_program::entrypoint_deprecated::ProgramResult {
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
