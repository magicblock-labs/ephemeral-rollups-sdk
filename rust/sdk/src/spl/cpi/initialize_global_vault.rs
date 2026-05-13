use crate::{
    compat::{self, Compat, Modern},
    consts::ESPL_TOKEN_PROGRAM_ID,
    spl::EphemeralSplDiscriminator,
};
use solana_program::program::{invoke, invoke_signed};

/// Create an initialize global vault instruction.
pub struct InitializeGlobalVault<'a> {
    pub payer: compat::AccountInfo<'a>,
    pub vault: compat::AccountInfo<'a>,
    pub mint: compat::AccountInfo<'a>,
    pub vault_ephemeral_ata: compat::AccountInfo<'a>,
    pub vault_ata: compat::AccountInfo<'a>,
    pub token_program: compat::AccountInfo<'a>,
    pub associated_token_program: compat::AccountInfo<'a>,
    pub system_program: compat::AccountInfo<'a>,
}

impl<'a> InitializeGlobalVault<'a> {
    #[inline(always)]
    pub fn instruction(&self) -> compat::Instruction {
        compat::Instruction {
            program_id: ESPL_TOKEN_PROGRAM_ID,
            accounts: vec![
                compat::AccountMeta::new(*self.vault.key, false),
                compat::AccountMeta::new(*self.payer.key, true),
                compat::AccountMeta::new_readonly(*self.mint.key, false),
                compat::AccountMeta::new(*self.vault_ephemeral_ata.key, false),
                compat::AccountMeta::new(*self.vault_ata.key, false),
                compat::AccountMeta::new_readonly(*self.token_program.key, false),
                compat::AccountMeta::new_readonly(*self.associated_token_program.key, false),
                compat::AccountMeta::new_readonly(*self.system_program.key, false),
            ],
            data: vec![EphemeralSplDiscriminator::InitializeGlobalVault as u8],
        }
    }

    #[inline(always)]
    pub fn invoke(&self) -> compat::ProgramResult {
        invoke(
            &self.instruction().modern(),
            &[
                self.vault.clone(),
                self.payer.clone(),
                self.mint.clone(),
                self.vault_ephemeral_ata.clone(),
                self.vault_ata.clone(),
                self.token_program.clone(),
                self.associated_token_program.clone(),
                self.system_program.clone(),
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
                self.vault.clone(),
                self.payer.clone(),
                self.mint.clone(),
                self.vault_ephemeral_ata.clone(),
                self.vault_ata.clone(),
                self.token_program.clone(),
                self.associated_token_program.clone(),
                self.system_program.clone(),
            ]
            .modern(),
            signers_seeds,
        )
        .compat()
    }
}
