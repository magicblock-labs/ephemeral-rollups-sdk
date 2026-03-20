use crate::{
    consts::ESPL_TOKEN_PROGRAM_ID,
    solana_compat::solana::{
        invoke, invoke_signed, AccountInfo, AccountMeta, Instruction, ProgramResult,
    },
    spl::EphemeralSplDiscriminator,
};

/// Delegate an ephemeral ATA permission.
pub struct DelegateEphemeralAtaPermission<'a> {
    pub payer: AccountInfo<'a>,
    pub eata: AccountInfo<'a>,
    pub permission_program: AccountInfo<'a>,
    pub permission: AccountInfo<'a>,
    pub system_program: AccountInfo<'a>,
    pub delegation_buffer: AccountInfo<'a>,
    pub delegation_record: AccountInfo<'a>,
    pub delegation_metadata: AccountInfo<'a>,
    pub delegation_program: AccountInfo<'a>,
    pub validator: AccountInfo<'a>,
    pub eata_bump: u8,
}

impl<'a> DelegateEphemeralAtaPermission<'a> {
    #[inline(always)]
    pub fn instruction(&self) -> Instruction {
        Instruction {
            program_id: ESPL_TOKEN_PROGRAM_ID,
            accounts: vec![
                AccountMeta::new(*self.payer.key, true),
                AccountMeta::new(*self.eata.key, false),
                AccountMeta::new_readonly(*self.permission_program.key, false),
                AccountMeta::new(*self.permission.key, false),
                AccountMeta::new_readonly(*self.system_program.key, false),
                AccountMeta::new(*self.delegation_buffer.key, false),
                AccountMeta::new(*self.delegation_record.key, false),
                AccountMeta::new(*self.delegation_metadata.key, false),
                AccountMeta::new_readonly(*self.delegation_program.key, false),
                AccountMeta::new_readonly(*self.validator.key, false),
            ],
            data: vec![EphemeralSplDiscriminator::DelegateEphemeralAtaPermission as u8],
        }
    }

    #[inline(always)]
    pub fn invoke(&self) -> ProgramResult {
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
    pub fn invoke_signed(&self, signers_seeds: &[&[&[u8]]]) -> ProgramResult {
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
