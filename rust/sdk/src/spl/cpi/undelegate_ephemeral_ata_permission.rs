use crate::{
    compat::{self, Compat, Modern},
    consts::ESPL_TOKEN_PROGRAM_ID,
    spl::EphemeralSplDiscriminator,
};
use solana_program::program::{invoke, invoke_signed};

/// Undelegate an ephemeral ATA permission.
pub struct UndelegateEphemeralAtaPermission<'a> {
    pub payer: compat::AccountInfo<'a>,
    pub eata: compat::AccountInfo<'a>,
    pub permission: compat::AccountInfo<'a>,
    pub permission_program: compat::AccountInfo<'a>,
    pub magic_program: compat::AccountInfo<'a>,
    pub magic_context: compat::AccountInfo<'a>,
}

impl<'a> UndelegateEphemeralAtaPermission<'a> {
    #[inline(always)]
    pub fn instruction(&self) -> compat::Instruction {
        compat::Instruction {
            program_id: ESPL_TOKEN_PROGRAM_ID,
            accounts: vec![
                compat::AccountMeta::new_readonly(*self.payer.key, true),
                compat::AccountMeta::new(*self.eata.key, false),
                compat::AccountMeta::new(*self.permission.key, false),
                compat::AccountMeta::new_readonly(*self.permission_program.key, false),
                compat::AccountMeta::new_readonly(*self.magic_program.key, false),
                compat::AccountMeta::new(*self.magic_context.key, false),
            ],
            data: vec![EphemeralSplDiscriminator::UndelegateEphemeralAtaPermission as u8],
        }
    }

    #[inline(always)]
    pub fn invoke(&self) -> compat::ProgramResult {
        invoke(
            &self.instruction().modern(),
            &[
                self.payer.clone(),
                self.eata.clone(),
                self.permission.clone(),
                self.permission_program.clone(),
                self.magic_program.clone(),
                self.magic_context.clone(),
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
                self.eata.clone(),
                self.permission.clone(),
                self.permission_program.clone(),
                self.magic_program.clone(),
                self.magic_context.clone(),
            ]
            .modern(),
            signers_seeds,
        )
        .compat()
    }
}
