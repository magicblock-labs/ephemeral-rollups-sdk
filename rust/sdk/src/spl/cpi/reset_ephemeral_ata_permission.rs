use crate::{
    compat::{self, Compat, Modern},
    consts::ESPL_TOKEN_PROGRAM_ID,
    spl::EphemeralSplDiscriminator,
};
use solana_program::program::{invoke, invoke_signed};

/// Build a reset ephemeral ATA permission instruction.
///
/// For details on the flag byte, see the [MemberFlags](`crate::access_control::structs::Member`) struct.
pub struct ResetEphemeralAtaPermission<'a> {
    pub eata: compat::AccountInfo<'a>,
    pub permission: compat::AccountInfo<'a>,
    pub owner: compat::AccountInfo<'a>,
    pub permission_program: compat::AccountInfo<'a>,
    pub flag_byte: u8,
}

impl<'a> ResetEphemeralAtaPermission<'a> {
    #[inline(always)]
    pub fn instruction(&self) -> compat::Instruction {
        compat::Instruction {
            program_id: ESPL_TOKEN_PROGRAM_ID,
            accounts: vec![
                compat::AccountMeta::new_readonly(*self.eata.key, false),
                compat::AccountMeta::new(*self.permission.key, false),
                compat::AccountMeta::new_readonly(*self.owner.key, true),
                compat::AccountMeta::new_readonly(*self.permission_program.key, false),
            ],
            data: vec![
                EphemeralSplDiscriminator::ResetEphemeralAtaPermission as u8,
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
                self.owner.clone(),
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
                self.owner.clone(),
                self.permission_program.clone(),
            ]
            .modern(),
            signers_seeds,
        )
        .compat()
    }
}
