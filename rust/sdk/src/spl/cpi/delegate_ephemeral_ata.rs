use crate::{consts::ESPL_TOKEN_PROGRAM_ID, spl::EphemeralSplDiscriminator};
use solana_program::program::{invoke, invoke_signed};

/// Delegate an ephemeral ATA.
pub struct DelegateEphemeralAta<'a> {
    pub payer: solana_program::account_info::AccountInfo<'a>,
    pub eata: solana_program::account_info::AccountInfo<'a>,
    pub espl_token_program: solana_program::account_info::AccountInfo<'a>,
    pub delegation_buffer: solana_program::account_info::AccountInfo<'a>,
    pub delegation_record: solana_program::account_info::AccountInfo<'a>,
    pub delegation_metadata: solana_program::account_info::AccountInfo<'a>,
    pub delegation_program: solana_program::account_info::AccountInfo<'a>,
    pub system_program: solana_program::account_info::AccountInfo<'a>,
    pub validator: Option<solana_program::pubkey::Pubkey>,
}

impl<'a> DelegateEphemeralAta<'a> {
    #[inline(always)]
    pub fn instruction(&self) -> solana_program::instruction::Instruction {
        let mut data = Vec::with_capacity(33);
        data.push(EphemeralSplDiscriminator::DelegateEphemeralAta as u8);
        if let Some(validator) = self.validator {
            data.extend_from_slice(validator.to_bytes().as_ref());
        }
        solana_program::instruction::Instruction {
            program_id: ESPL_TOKEN_PROGRAM_ID,
            accounts: vec![
                solana_program::instruction::AccountMeta::new(*self.payer.key, true),
                solana_program::instruction::AccountMeta::new(*self.eata.key, false),
                solana_program::instruction::AccountMeta::new_readonly(
                    *self.espl_token_program.key,
                    false,
                ),
                solana_program::instruction::AccountMeta::new(*self.delegation_buffer.key, false),
                solana_program::instruction::AccountMeta::new(*self.delegation_record.key, false),
                solana_program::instruction::AccountMeta::new(*self.delegation_metadata.key, false),
                solana_program::instruction::AccountMeta::new_readonly(
                    *self.delegation_program.key,
                    false,
                ),
                solana_program::instruction::AccountMeta::new_readonly(
                    *self.system_program.key,
                    false,
                ),
            ],
            data,
        }
    }

    #[inline(always)]
    pub fn invoke(&self) -> solana_program::entrypoint_deprecated::ProgramResult {
        invoke(
            &self.instruction(),
            &[
                self.payer.clone(),
                self.eata.clone(),
                self.espl_token_program.clone(),
                self.delegation_buffer.clone(),
                self.delegation_record.clone(),
                self.delegation_metadata.clone(),
                self.delegation_program.clone(),
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
                self.payer.clone(),
                self.eata.clone(),
                self.espl_token_program.clone(),
                self.delegation_buffer.clone(),
                self.delegation_record.clone(),
                self.delegation_metadata.clone(),
                self.delegation_program.clone(),
                self.system_program.clone(),
            ],
            signers_seeds,
        )
    }
}
