use crate::{
    compat::{self, Compat, Modern},
    consts::ESPL_TOKEN_PROGRAM_ID,
    spl::EphemeralSplDiscriminator,
};
use solana_program::program::{invoke, invoke_signed};

/// Create a new ephemeral ATA permission.
///
/// For details on the flag byte, see the [MemberFlags](`crate::access_control::structs::Member`) struct.
pub struct CreateEphemeralAtaPermission<'a> {
    pub eata: compat::AccountInfo<'a>,
    pub permission: compat::AccountInfo<'a>,
    pub payer: compat::AccountInfo<'a>,
    pub system_program: compat::AccountInfo<'a>,
    pub permission_program: compat::AccountInfo<'a>,
    pub flag_byte: u8,
}

impl<'a> CreateEphemeralAtaPermission<'a> {
    #[inline(always)]
    pub fn instruction(&self) -> compat::Instruction {
        compat::Instruction {
            program_id: ESPL_TOKEN_PROGRAM_ID,
            accounts: vec![
                compat::AccountMeta::new(*self.eata.key, false),
                compat::AccountMeta::new(*self.permission.key, false),
                compat::AccountMeta::new(*self.payer.key, true),
                compat::AccountMeta::new_readonly(*self.system_program.key, false),
                compat::AccountMeta::new_readonly(*self.permission_program.key, false),
            ],
            data: vec![
                EphemeralSplDiscriminator::CreateEphemeralAtaPermission as u8,
                self.flag_byte,
            ],
        }
    }

    #[inline(always)]
    pub fn invoke(&self) -> compat::ProgramResult {
        invoke(
            &self.instruction().modern(),
            &[
                self.eata.clone(),
                self.permission.clone(),
                self.payer.clone(),
                self.system_program.clone(),
                self.permission_program.clone(),
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
                self.permission.clone(),
                self.payer.clone(),
                self.system_program.clone(),
                self.permission_program.clone(),
            ]
            .modern(),
            signers_seeds,
        )
        .compat()
    }
}
