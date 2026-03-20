use crate::{
    consts::ESPL_TOKEN_PROGRAM_ID,
    solana_compat::solana::{
        invoke, invoke_signed, AccountInfo, AccountMeta, Instruction, ProgramResult,
    },
    spl::EphemeralSplDiscriminator,
};

/// Create an initialize global vault instruction.
pub struct InitializeGlobalVault<'a> {
    pub payer: AccountInfo<'a>,
    pub vault: AccountInfo<'a>,
    pub mint: AccountInfo<'a>,
    pub vault_ephemeral_ata: AccountInfo<'a>,
    pub vault_ata: AccountInfo<'a>,
    pub token_program: AccountInfo<'a>,
    pub associated_token_program: AccountInfo<'a>,
    pub system_program: AccountInfo<'a>,
    pub vault_bump: u8,
}

impl<'a> InitializeGlobalVault<'a> {
    #[inline(always)]
    pub fn instruction(&self) -> Instruction {
        Instruction {
            program_id: ESPL_TOKEN_PROGRAM_ID,
            accounts: vec![
                AccountMeta::new(*self.vault.key, false),
                AccountMeta::new(*self.payer.key, false),
                AccountMeta::new_readonly(*self.mint.key, false),
                AccountMeta::new(*self.vault_ephemeral_ata.key, false),
                AccountMeta::new(*self.vault_ata.key, false),
                AccountMeta::new_readonly(*self.token_program.key, false),
                AccountMeta::new_readonly(*self.associated_token_program.key, false),
                AccountMeta::new_readonly(*self.system_program.key, false),
            ],
            data: vec![
                EphemeralSplDiscriminator::InitializeGlobalVault as u8,
                self.vault_bump,
            ],
        }
    }

    #[inline(always)]
    pub fn invoke(&self) -> ProgramResult {
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
    pub fn invoke_signed(&self, signers_seeds: &[&[&[u8]]]) -> ProgramResult {
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
