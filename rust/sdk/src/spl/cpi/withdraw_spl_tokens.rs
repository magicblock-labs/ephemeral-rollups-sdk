use crate::{consts::ESPL_TOKEN_PROGRAM_ID, spl::EphemeralSplDiscriminator};
use solana_program::program::{invoke, invoke_signed};

/// Build a withdraw SPL tokens instruction.
pub struct WithdrawSplTokens<'a> {
    pub payer: solana_program::account_info::AccountInfo<'a>,
    pub eata: solana_program::account_info::AccountInfo<'a>,
    pub vault: solana_program::account_info::AccountInfo<'a>,
    pub mint: solana_program::account_info::AccountInfo<'a>,
    pub vault_ata: solana_program::account_info::AccountInfo<'a>,
    pub user_ata: solana_program::account_info::AccountInfo<'a>,
    pub token_program: solana_program::account_info::AccountInfo<'a>,
    pub vault_bump: u8,
    pub amount: u64,
}

impl<'a> WithdrawSplTokens<'a> {
    #[inline(always)]
    pub fn instruction(&self) -> solana_program::instruction::Instruction {
        let mut data = Vec::with_capacity(9);
        data.push(EphemeralSplDiscriminator::WithdrawSplTokens as u8);
        data.extend_from_slice(self.amount.to_le_bytes().as_ref());
        solana_program::instruction::Instruction {
            program_id: ESPL_TOKEN_PROGRAM_ID,
            accounts: vec![
                solana_program::instruction::AccountMeta::new(*self.eata.key, false),
                solana_program::instruction::AccountMeta::new_readonly(*self.vault.key, false),
                solana_program::instruction::AccountMeta::new_readonly(*self.mint.key, false),
                solana_program::instruction::AccountMeta::new(*self.vault_ata.key, false),
                solana_program::instruction::AccountMeta::new(*self.user_ata.key, false),
                solana_program::instruction::AccountMeta::new_readonly(*self.payer.key, true),
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
                self.vault_ata.clone(),
                self.user_ata.clone(),
                self.payer.clone(),
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
                self.vault_ata.clone(),
                self.user_ata.clone(),
                self.payer.clone(),
                self.token_program.clone(),
            ],
            signers_seeds,
        )
    }
}
