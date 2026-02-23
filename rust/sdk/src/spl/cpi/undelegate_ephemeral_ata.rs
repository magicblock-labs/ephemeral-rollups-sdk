use crate::{
    consts::ESPL_TOKEN_PROGRAM_ID,
    solana_compat::solana::{
        invoke, invoke_signed, AccountInfo, AccountMeta, Instruction, ProgramResult,
    },
    spl::EphemeralSplDiscriminator,
};

/// Build an undelegate ephemeral ATA instruction.
pub struct UndelegateEphemeralAta<'a> {
    pub payer: AccountInfo<'a>,
    pub user_ata: AccountInfo<'a>,
    pub eata: AccountInfo<'a>,
    pub magic_context: AccountInfo<'a>,
    pub magic_program: AccountInfo<'a>,
}

impl<'a> UndelegateEphemeralAta<'a> {
    #[inline(always)]
    pub fn instruction(&self) -> Instruction {
        Instruction {
            program_id: ESPL_TOKEN_PROGRAM_ID,
            accounts: vec![
                AccountMeta::new_readonly(*self.payer.key, true),
                AccountMeta::new(*self.user_ata.key, false),
                AccountMeta::new_readonly(*self.eata.key, false),
                AccountMeta::new(*self.magic_context.key, false),
                AccountMeta::new_readonly(*self.magic_program.key, false),
            ],
            data: vec![EphemeralSplDiscriminator::UndelegateEphemeralAta as u8],
        }
    }

    #[inline(always)]
    pub fn invoke(&self) -> ProgramResult {
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
    pub fn invoke_signed(&self, signers_seeds: &[&[&[u8]]]) -> ProgramResult {
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
