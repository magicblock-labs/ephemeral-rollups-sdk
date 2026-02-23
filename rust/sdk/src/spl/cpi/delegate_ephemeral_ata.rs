use crate::{
    consts::ESPL_TOKEN_PROGRAM_ID,
    solana_compat::solana::{
        invoke, invoke_signed, AccountInfo, AccountMeta, Instruction, ProgramResult, Pubkey,
    },
    spl::EphemeralSplDiscriminator,
};

/// Delegate an ephemeral ATA.
pub struct DelegateEphemeralAta<'a> {
    pub payer: AccountInfo<'a>,
    pub eata: AccountInfo<'a>,
    pub espl_token_program: AccountInfo<'a>,
    pub delegation_buffer: AccountInfo<'a>,
    pub delegation_record: AccountInfo<'a>,
    pub delegation_metadata: AccountInfo<'a>,
    pub delegation_program: AccountInfo<'a>,
    pub system_program: AccountInfo<'a>,
    pub eata_bump: u8,
    pub validator: Option<Pubkey>,
}

impl<'a> DelegateEphemeralAta<'a> {
    #[inline(always)]
    pub fn instruction(&self) -> Instruction {
        let mut data = Vec::with_capacity(34);
        data.push(EphemeralSplDiscriminator::DelegateEphemeralAta as u8);
        data.push(self.eata_bump);
        if let Some(validator) = self.validator {
            data.extend_from_slice(validator.to_bytes().as_ref());
        }
        Instruction {
            program_id: ESPL_TOKEN_PROGRAM_ID,
            accounts: vec![
                AccountMeta::new(*self.payer.key, true),
                AccountMeta::new(*self.eata.key, false),
                AccountMeta::new_readonly(*self.espl_token_program.key, false),
                AccountMeta::new(*self.delegation_buffer.key, false),
                AccountMeta::new(*self.delegation_record.key, false),
                AccountMeta::new(*self.delegation_metadata.key, false),
                AccountMeta::new_readonly(*self.delegation_program.key, false),
                AccountMeta::new_readonly(*self.system_program.key, false),
            ],
            data,
        }
    }

    #[inline(always)]
    pub fn invoke(&self) -> ProgramResult {
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
    pub fn invoke_signed(&self, signers_seeds: &[&[&[u8]]]) -> ProgramResult {
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
