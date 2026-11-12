use crate::{consts::ESPL_TOKEN_PROGRAM_ID, spl::EphemeralSplDiscriminator};
use solana_program::program::{invoke, invoke_signed};

/// Delegate an ephemeral ATA permission.
pub struct DelegateEphemeralAtaPermission<'a> {
    pub payer: solana_program::account_info::AccountInfo<'a>,
    pub eata: solana_program::account_info::AccountInfo<'a>,
    pub permission_program: solana_program::account_info::AccountInfo<'a>,
    pub permission: solana_program::account_info::AccountInfo<'a>,
    pub system_program: solana_program::account_info::AccountInfo<'a>,
    pub delegation_buffer: solana_program::account_info::AccountInfo<'a>,
    pub delegation_record: solana_program::account_info::AccountInfo<'a>,
    pub delegation_metadata: solana_program::account_info::AccountInfo<'a>,
    pub delegation_program: solana_program::account_info::AccountInfo<'a>,
    pub validator: solana_program::account_info::AccountInfo<'a>,
    pub eata_bump: u8,
}

impl<'a> DelegateEphemeralAtaPermission<'a> {
    #[inline(always)]
    pub fn instruction(&self) -> solana_program::instruction::Instruction {
        solana_program::instruction::Instruction {
            program_id: ESPL_TOKEN_PROGRAM_ID,
            accounts: vec![
                solana_program::instruction::AccountMeta::new(*self.payer.key, true),
                solana_program::instruction::AccountMeta::new(*self.eata.key, false),
                solana_program::instruction::AccountMeta::new_readonly(
                    *self.permission_program.key,
                    false,
                ),
                solana_program::instruction::AccountMeta::new(*self.permission.key, false),
                solana_program::instruction::AccountMeta::new_readonly(
                    *self.system_program.key,
                    false,
                ),
                solana_program::instruction::AccountMeta::new(*self.delegation_buffer.key, false),
                solana_program::instruction::AccountMeta::new(*self.delegation_record.key, false),
                solana_program::instruction::AccountMeta::new(*self.delegation_metadata.key, false),
                solana_program::instruction::AccountMeta::new_readonly(
                    *self.delegation_program.key,
                    false,
                ),
                solana_program::instruction::AccountMeta::new_readonly(*self.validator.key, false),
            ],
            data: vec![EphemeralSplDiscriminator::DelegateEphemeralAtaPermission as u8],
        }
    }

    #[inline(always)]
    pub fn invoke(&self) -> solana_program::entrypoint_deprecated::ProgramResult {
        invoke(
            &self.instruction(),
            &[
                self.payer.clone(),
                self.eata.clone(),
                self.permission_program.clone(),
                self.permission.clone(),
                self.system_program.clone(),
                self.delegation_buffer.clone(),
                self.delegation_record.clone(),
                self.delegation_metadata.clone(),
                self.delegation_program.clone(),
                self.validator.clone(),
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
                self.permission_program.clone(),
                self.permission.clone(),
                self.system_program.clone(),
                self.delegation_buffer.clone(),
                self.delegation_record.clone(),
                self.delegation_metadata.clone(),
                self.delegation_program.clone(),
                self.validator.clone(),
            ],
            signers_seeds,
        )
    }
}
