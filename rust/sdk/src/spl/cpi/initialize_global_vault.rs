use crate::{consts::ESPL_TOKEN_PROGRAM_ID, spl::EphemeralSplDiscriminator};
use solana_program::program::{invoke, invoke_signed};

/// Create an initialize global vault instruction.
pub struct InitializeGlobalVault<'a> {
    pub payer: solana_program::account_info::AccountInfo<'a>,
    pub vault: solana_program::account_info::AccountInfo<'a>,
    pub mint: solana_program::account_info::AccountInfo<'a>,
    pub vault_ephemeral_ata: solana_program::account_info::AccountInfo<'a>,
    pub vault_ata: solana_program::account_info::AccountInfo<'a>,
    pub token_program: solana_program::account_info::AccountInfo<'a>,
    pub associated_token_program: solana_program::account_info::AccountInfo<'a>,
    pub system_program: solana_program::account_info::AccountInfo<'a>,
}

impl<'a> InitializeGlobalVault<'a> {
    #[inline(always)]
    pub fn instruction(&self) -> solana_program::instruction::Instruction {
        solana_program::instruction::Instruction {
            program_id: ESPL_TOKEN_PROGRAM_ID,
            accounts: vec![
                solana_program::instruction::AccountMeta::new(*self.vault.key, false),
                solana_program::instruction::AccountMeta::new(*self.payer.key, true),
                solana_program::instruction::AccountMeta::new_readonly(*self.mint.key, false),
                solana_program::instruction::AccountMeta::new(*self.vault_ephemeral_ata.key, false),
                solana_program::instruction::AccountMeta::new(*self.vault_ata.key, false),
                solana_program::instruction::AccountMeta::new_readonly(
                    *self.token_program.key,
                    false,
                ),
                solana_program::instruction::AccountMeta::new_readonly(
                    *self.associated_token_program.key,
                    false,
                ),
                solana_program::instruction::AccountMeta::new_readonly(
                    *self.system_program.key,
                    false,
                ),
            ],
            data: vec![EphemeralSplDiscriminator::InitializeGlobalVault as u8],
        }
    }

    #[inline(always)]
    pub fn invoke(&self) -> solana_program::entrypoint_deprecated::ProgramResult {
        invoke(
            &self.instruction(),
            &[
                self.vault.clone(),
                self.payer.clone(),
                self.mint.clone(),
                self.vault_ephemeral_ata.clone(),
                self.vault_ata.clone(),
                self.token_program.clone(),
                self.associated_token_program.clone(),
                self.system_program.clone(),
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
                self.vault.clone(),
                self.payer.clone(),
                self.mint.clone(),
                self.vault_ephemeral_ata.clone(),
                self.vault_ata.clone(),
                self.token_program.clone(),
                self.associated_token_program.clone(),
                self.system_program.clone(),
            ],
            signers_seeds,
        )
    }
}
