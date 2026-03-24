use crate::{
    consts::ESPL_TOKEN_PROGRAM_ID,
    solana_compat::solana::{
        invoke, invoke_signed, AccountInfo, AccountMeta, Instruction, ProgramResult,
    },
    spl::EphemeralSplDiscriminator,
};

/// Create an initialize ephemeral ATA instruction.
pub struct InitializeEphemeralAta<'a> {
    pub payer: AccountInfo<'a>,
    pub eata: AccountInfo<'a>,
    pub user: AccountInfo<'a>,
    pub mint: AccountInfo<'a>,
    pub system_program: AccountInfo<'a>,
}

impl<'a> InitializeEphemeralAta<'a> {
    #[inline(always)]
    pub fn instruction(&self) -> Instruction {
        Instruction {
            program_id: ESPL_TOKEN_PROGRAM_ID,
            accounts: vec![
                AccountMeta::new(*self.eata.key, false),
                AccountMeta::new(*self.payer.key, false),
                AccountMeta::new_readonly(*self.user.key, false),
                AccountMeta::new_readonly(*self.mint.key, false),
                AccountMeta::new_readonly(*self.system_program.key, false),
            ],
            data: vec![EphemeralSplDiscriminator::InitializeEphemeralAta as u8],
        }
    }

    #[inline(always)]
    pub fn invoke(&self) -> ProgramResult {
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
    pub fn invoke_signed(&self, signers_seeds: &[&[&[u8]]]) -> ProgramResult {
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
