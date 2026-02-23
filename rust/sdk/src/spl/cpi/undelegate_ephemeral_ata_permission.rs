use crate::{
    consts::ESPL_TOKEN_PROGRAM_ID,
    solana_compat::solana::{
        invoke, invoke_signed, AccountInfo, AccountMeta, Instruction, ProgramResult,
    },
    spl::EphemeralSplDiscriminator,
};

/// Undelegate an ephemeral ATA permission.
pub struct UndelegateEphemeralAtaPermission<'a> {
    pub payer: AccountInfo<'a>,
    pub eata: AccountInfo<'a>,
    pub permission: AccountInfo<'a>,
    pub permission_program: AccountInfo<'a>,
    pub magic_program: AccountInfo<'a>,
    pub magic_context: AccountInfo<'a>,
}

impl<'a> UndelegateEphemeralAtaPermission<'a> {
    #[inline(always)]
    pub fn instruction(&self) -> Instruction {
        Instruction {
            program_id: ESPL_TOKEN_PROGRAM_ID,
            accounts: vec![
                AccountMeta::new_readonly(*self.payer.key, true),
                AccountMeta::new_readonly(*self.eata.key, false),
                AccountMeta::new(*self.permission.key, false),
                AccountMeta::new_readonly(*self.permission_program.key, false),
                AccountMeta::new_readonly(*self.magic_program.key, false),
                AccountMeta::new(*self.magic_context.key, false),
            ],
            data: vec![EphemeralSplDiscriminator::UndelegateEphemeralAtaPermission as u8],
        }
    }

    #[inline(always)]
    pub fn invoke(&self) -> ProgramResult {
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
    pub fn invoke_signed(&self, signers_seeds: &[&[&[u8]]]) -> ProgramResult {
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
