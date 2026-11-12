use crate::{consts::ESPL_TOKEN_PROGRAM_ID, spl::EphemeralSplDiscriminator};
use solana_program::program::{invoke, invoke_signed};

/// Undelegate an ephemeral ATA permission.
pub struct UndelegateEphemeralAtaPermission<'a> {
    pub payer: solana_program::account_info::AccountInfo<'a>,
    pub eata: solana_program::account_info::AccountInfo<'a>,
    pub permission: solana_program::account_info::AccountInfo<'a>,
    pub permission_program: solana_program::account_info::AccountInfo<'a>,
    pub magic_program: solana_program::account_info::AccountInfo<'a>,
    pub magic_context: solana_program::account_info::AccountInfo<'a>,
}

impl<'a> UndelegateEphemeralAtaPermission<'a> {
    #[inline(always)]
    pub fn instruction(&self) -> solana_program::instruction::Instruction {
        solana_program::instruction::Instruction {
            program_id: ESPL_TOKEN_PROGRAM_ID,
            accounts: vec![
                solana_program::instruction::AccountMeta::new_readonly(*self.payer.key, true),
                solana_program::instruction::AccountMeta::new(*self.eata.key, false),
                solana_program::instruction::AccountMeta::new(*self.permission.key, false),
                solana_program::instruction::AccountMeta::new_readonly(
                    *self.permission_program.key,
                    false,
                ),
                solana_program::instruction::AccountMeta::new_readonly(
                    *self.magic_program.key,
                    false,
                ),
                solana_program::instruction::AccountMeta::new(*self.magic_context.key, false),
            ],
            data: vec![EphemeralSplDiscriminator::UndelegateEphemeralAtaPermission as u8],
        }
    }

    #[inline(always)]
    pub fn invoke(&self) -> solana_program::entrypoint_deprecated::ProgramResult {
        invoke(
            &self.instruction(),
            &[
                self.payer.clone(),
                self.eata.clone(),
                self.permission.clone(),
                self.permission_program.clone(),
                self.magic_program.clone(),
                self.magic_context.clone(),
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
                self.payer.clone(),
                self.eata.clone(),
                self.permission.clone(),
                self.permission_program.clone(),
                self.magic_program.clone(),
                self.magic_context.clone(),
            ],
            signers_seeds,
        )
    }
}
