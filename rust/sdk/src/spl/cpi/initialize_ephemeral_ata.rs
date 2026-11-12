use crate::{consts::ESPL_TOKEN_PROGRAM_ID, spl::EphemeralSplDiscriminator};
use solana_program::program::{invoke, invoke_signed};

/// Create an initialize ephemeral ATA instruction.
pub struct InitializeEphemeralAta<'a> {
    pub payer: solana_program::account_info::AccountInfo<'a>,
    pub eata: solana_program::account_info::AccountInfo<'a>,
    pub user: solana_program::account_info::AccountInfo<'a>,
    pub mint: solana_program::account_info::AccountInfo<'a>,
    pub system_program: solana_program::account_info::AccountInfo<'a>,
}

impl<'a> InitializeEphemeralAta<'a> {
    #[inline(always)]
    pub fn instruction(&self) -> solana_program::instruction::Instruction {
        solana_program::instruction::Instruction {
            program_id: ESPL_TOKEN_PROGRAM_ID,
            accounts: vec![
                solana_program::instruction::AccountMeta::new(*self.eata.key, false),
                solana_program::instruction::AccountMeta::new(*self.payer.key, false),
                solana_program::instruction::AccountMeta::new_readonly(*self.user.key, false),
                solana_program::instruction::AccountMeta::new_readonly(*self.mint.key, false),
                solana_program::instruction::AccountMeta::new_readonly(
                    *self.system_program.key,
                    false,
                ),
            ],
            data: vec![EphemeralSplDiscriminator::InitializeEphemeralAta as u8],
        }
    }

    #[inline(always)]
    pub fn invoke(&self) -> solana_program::entrypoint_deprecated::ProgramResult {
        invoke(
            &self.instruction(),
            &[
                self.eata.clone(),
                self.payer.clone(),
                self.user.clone(),
                self.mint.clone(),
                self.system_program.clone(),
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
                self.payer.clone(),
                self.user.clone(),
                self.mint.clone(),
                self.system_program.clone(),
            ],
            signers_seeds,
        )
    }
}
