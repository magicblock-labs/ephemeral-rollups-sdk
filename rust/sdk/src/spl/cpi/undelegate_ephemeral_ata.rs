use crate::{consts::ESPL_TOKEN_PROGRAM_ID, spl::EphemeralSplDiscriminator};
use solana_program::program::{invoke, invoke_signed};

/// Build an undelegate ephemeral ATA instruction.
pub struct UndelegateEphemeralAta<'a> {
    pub payer: solana_program::account_info::AccountInfo<'a>,
    pub user_ata: solana_program::account_info::AccountInfo<'a>,
    pub eata: solana_program::account_info::AccountInfo<'a>,
    pub magic_context: solana_program::account_info::AccountInfo<'a>,
    pub magic_program: solana_program::account_info::AccountInfo<'a>,
}

impl<'a> UndelegateEphemeralAta<'a> {
    #[inline(always)]
    pub fn instruction(&self) -> solana_program::instruction::Instruction {
        solana_program::instruction::Instruction {
            program_id: ESPL_TOKEN_PROGRAM_ID,
            accounts: vec![
                solana_program::instruction::AccountMeta::new_readonly(*self.payer.key, true),
                solana_program::instruction::AccountMeta::new(*self.user_ata.key, false),
                solana_program::instruction::AccountMeta::new_readonly(*self.eata.key, false),
                solana_program::instruction::AccountMeta::new(*self.magic_context.key, false),
                solana_program::instruction::AccountMeta::new_readonly(
                    *self.magic_program.key,
                    false,
                ),
            ],
            data: vec![EphemeralSplDiscriminator::UndelegateEphemeralAta as u8],
        }
    }

    #[inline(always)]
    pub fn invoke(&self) -> solana_program::entrypoint_deprecated::ProgramResult {
        invoke(
            &self.instruction(),
            &[
                self.payer.clone(),
                self.user_ata.clone(),
                self.eata.clone(),
                self.magic_context.clone(),
                self.magic_program.clone(),
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
                self.user_ata.clone(),
                self.eata.clone(),
                self.magic_context.clone(),
                self.magic_program.clone(),
            ],
            signers_seeds,
        )
    }
}
