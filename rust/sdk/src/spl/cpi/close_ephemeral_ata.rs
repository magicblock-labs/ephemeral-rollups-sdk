use crate::{
    consts::ESPL_TOKEN_PROGRAM_ID,
    solana_compat::solana::{
        invoke, invoke_signed, AccountInfo, AccountMeta, Instruction, ProgramResult,
    },
    spl::EphemeralSplDiscriminator,
};

/// Create an initialize ephemeral ATA instruction.
pub struct CloseEphemeralAta<'a> {
    pub user: AccountInfo<'a>,
    pub eata: AccountInfo<'a>,
    pub payer: AccountInfo<'a>,
}

impl<'a> CloseEphemeralAta<'a> {
    #[inline(always)]
    pub fn instruction(&self) -> Instruction {
        Instruction {
            program_id: ESPL_TOKEN_PROGRAM_ID,
            accounts: vec![
                AccountMeta::new_readonly(*self.user.key, true),
                AccountMeta::new(*self.eata.key, false),
                AccountMeta::new(*self.payer.key, false),
            ],
            data: vec![EphemeralSplDiscriminator::CloseEphemeralAta as u8],
        }
    }

    #[inline(always)]
    pub fn invoke(&self) -> ProgramResult {
        invoke(
            &self.instruction(),
            &[self.user.clone(), self.eata.clone(), self.payer.clone()],
        )
    }

    #[inline(always)]
    pub fn invoke_signed(&self, signers_seeds: &[&[&[u8]]]) -> ProgramResult {
        invoke_signed(
            &self.instruction(),
            &[self.user.clone(), self.eata.clone(), self.payer.clone()],
            signers_seeds,
        )
    }
}
