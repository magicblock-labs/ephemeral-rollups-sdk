use crate::{consts::ESPL_TOKEN_PROGRAM_ID, spl::EphemeralSplDiscriminator};
use solana_program::program::{invoke, invoke_signed};

/// Build a reset ephemeral ATA permission instruction.
///
/// For details on the flag byte, see the [MemberFlags](`crate::access_control::structs::Member`) struct.
pub struct ResetEphemeralAtaPermission<'a> {
    pub eata: solana_program::account_info::AccountInfo<'a>,
    pub permission: solana_program::account_info::AccountInfo<'a>,
    pub owner: solana_program::account_info::AccountInfo<'a>,
    pub permission_program: solana_program::account_info::AccountInfo<'a>,
    pub flag_byte: u8,
}

impl<'a> ResetEphemeralAtaPermission<'a> {
    #[inline(always)]
    pub fn instruction(&self) -> solana_program::instruction::Instruction {
        solana_program::instruction::Instruction {
            program_id: ESPL_TOKEN_PROGRAM_ID,
            accounts: vec![
                solana_program::instruction::AccountMeta::new_readonly(*self.eata.key, false),
                solana_program::instruction::AccountMeta::new(*self.permission.key, false),
                solana_program::instruction::AccountMeta::new_readonly(*self.owner.key, true),
                solana_program::instruction::AccountMeta::new_readonly(
                    *self.permission_program.key,
                    false,
                ),
            ],
            data: vec![
                EphemeralSplDiscriminator::ResetEphemeralAtaPermission as u8,
                self.flag_byte,
            ],
        }
    }

    #[inline(always)]
    pub fn invoke(&self) -> solana_program::entrypoint_deprecated::ProgramResult {
        invoke(
            &self.instruction(),
            &[
                self.eata.clone(),
                self.permission.clone(),
                self.owner.clone(),
                self.permission_program.clone(),
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
                self.permission.clone(),
                self.owner.clone(),
                self.permission_program.clone(),
            ],
            signers_seeds,
        )
    }
}
