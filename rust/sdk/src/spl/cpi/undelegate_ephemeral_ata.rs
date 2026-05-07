use crate::{
    compat::{self, Compat, Modern},
    consts::ESPL_TOKEN_PROGRAM_ID,
    spl::EphemeralSplDiscriminator,
};
use solana_program::program::{invoke, invoke_signed};

/// Build an undelegate ephemeral ATA instruction.
pub struct UndelegateEphemeralAta<'a> {
    pub payer: compat::AccountInfo<'a>,
    pub user_ata: compat::AccountInfo<'a>,
    pub eata: compat::AccountInfo<'a>,
    pub magic_context: compat::AccountInfo<'a>,
    pub magic_program: compat::AccountInfo<'a>,
}

impl<'a> UndelegateEphemeralAta<'a> {
    #[inline(always)]
    pub fn instruction(&self) -> compat::Instruction {
        compat::Instruction {
            program_id: ESPL_TOKEN_PROGRAM_ID,
            accounts: vec![
                compat::AccountMeta::new_readonly(*self.payer.key, true),
                compat::AccountMeta::new(*self.user_ata.key, false),
                compat::AccountMeta::new_readonly(*self.eata.key, false),
                compat::AccountMeta::new(*self.magic_context.key, false),
                compat::AccountMeta::new_readonly(*self.magic_program.key, false),
            ],
            data: vec![EphemeralSplDiscriminator::UndelegateEphemeralAta as u8],
        }
    }

    #[inline(always)]
    pub fn invoke(&self) -> compat::ProgramResult {
        invoke(
            &self.instruction().modern(),
            &[
                self.payer.clone(),
                self.user_ata.clone(),
                self.eata.clone(),
                self.magic_context.clone(),
                self.magic_program.clone(),
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
                self.payer.clone(),
                self.user_ata.clone(),
                self.eata.clone(),
                self.magic_context.clone(),
                self.magic_program.clone(),
            ]
            .modern(),
            signers_seeds,
        )
        .compat()
    }
}
