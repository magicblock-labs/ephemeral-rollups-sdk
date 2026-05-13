use crate::{
    compat::{self, Compat, Modern},
    consts::ESPL_TOKEN_PROGRAM_ID,
    spl::EphemeralSplDiscriminator,
};
use solana_program::program::{invoke, invoke_signed};

/// Create an initialize ephemeral ATA instruction.
pub struct InitializeEphemeralAta<'a> {
    pub payer: compat::AccountInfo<'a>,
    pub eata: compat::AccountInfo<'a>,
    pub user: compat::AccountInfo<'a>,
    pub mint: compat::AccountInfo<'a>,
    pub system_program: compat::AccountInfo<'a>,
}

impl<'a> InitializeEphemeralAta<'a> {
    #[inline(always)]
    pub fn instruction(&self) -> compat::Instruction {
        compat::Instruction {
            program_id: ESPL_TOKEN_PROGRAM_ID,
            accounts: vec![
                compat::AccountMeta::new(*self.eata.key, false),
                compat::AccountMeta::new(*self.payer.key, false),
                compat::AccountMeta::new_readonly(*self.user.key, false),
                compat::AccountMeta::new_readonly(*self.mint.key, false),
                compat::AccountMeta::new_readonly(*self.system_program.key, false),
            ],
            data: vec![EphemeralSplDiscriminator::InitializeEphemeralAta as u8],
        }
    }

    #[inline(always)]
    pub fn invoke(&self) -> compat::ProgramResult {
        invoke(
            &self.instruction().modern(),
            &[
                self.eata.clone(),
                self.payer.clone(),
                self.user.clone(),
                self.mint.clone(),
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
                self.eata.clone(),
                self.payer.clone(),
                self.user.clone(),
                self.mint.clone(),
                self.system_program.clone(),
            ]
            .modern(),
            signers_seeds,
        )
        .compat()
    }
}
