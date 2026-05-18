use crate::{
    compat::{self, Compat, Modern},
    consts::ESPL_TOKEN_PROGRAM_ID,
    spl::EphemeralSplDiscriminator,
};
use solana_program::program::{invoke, invoke_signed};

/// Delegate an ephemeral ATA.
pub struct DelegateEphemeralAta<'a> {
    pub payer: compat::AccountInfo<'a>,
    pub eata: compat::AccountInfo<'a>,
    pub espl_token_program: compat::AccountInfo<'a>,
    pub delegation_buffer: compat::AccountInfo<'a>,
    pub delegation_record: compat::AccountInfo<'a>,
    pub delegation_metadata: compat::AccountInfo<'a>,
    pub delegation_program: compat::AccountInfo<'a>,
    pub system_program: compat::AccountInfo<'a>,
    pub validator: Option<compat::Pubkey>,
}

impl<'a> DelegateEphemeralAta<'a> {
    #[inline(always)]
    pub fn instruction(&self) -> compat::Instruction {
        let mut data = Vec::with_capacity(33);
        data.push(EphemeralSplDiscriminator::DelegateEphemeralAta as u8);
        if let Some(validator) = self.validator {
            data.extend_from_slice(validator.to_bytes().as_ref());
        }
        compat::Instruction {
            program_id: ESPL_TOKEN_PROGRAM_ID,
            accounts: vec![
                compat::AccountMeta::new(*self.payer.key, true),
                compat::AccountMeta::new(*self.eata.key, false),
                compat::AccountMeta::new_readonly(*self.espl_token_program.key, false),
                compat::AccountMeta::new(*self.delegation_buffer.key, false),
                compat::AccountMeta::new(*self.delegation_record.key, false),
                compat::AccountMeta::new(*self.delegation_metadata.key, false),
                compat::AccountMeta::new_readonly(*self.delegation_program.key, false),
                compat::AccountMeta::new_readonly(*self.system_program.key, false),
            ],
            data,
        }
    }

    #[inline(always)]
    pub fn invoke(&self) -> compat::ProgramResult {
        invoke(
            &self.instruction().modern(),
            &[
                self.payer.clone(),
                self.eata.clone(),
                self.espl_token_program.clone(),
                self.delegation_buffer.clone(),
                self.delegation_record.clone(),
                self.delegation_metadata.clone(),
                self.delegation_program.clone(),
                self.system_program.clone(),
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
                self.espl_token_program.clone(),
                self.delegation_buffer.clone(),
                self.delegation_record.clone(),
                self.delegation_metadata.clone(),
                self.delegation_program.clone(),
                self.system_program.clone(),
            ]
            .modern(),
            signers_seeds,
        )
        .compat()
    }
}
