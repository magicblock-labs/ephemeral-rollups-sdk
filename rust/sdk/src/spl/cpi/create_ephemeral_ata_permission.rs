use crate::{
    consts::ESPL_TOKEN_PROGRAM_ID,
    solana_compat::solana::{
        invoke, invoke_signed, AccountInfo, AccountMeta, Instruction, ProgramResult,
    },
    spl::EphemeralSplDiscriminator,
};

/// Create a new ephemeral ATA permission.
///
/// For details on the flag byte, see the [MemberFlags](`crate::access_control::structs::Member`) struct.
pub struct CreateEphemeralAtaPermission<'a> {
    pub eata: AccountInfo<'a>,
    pub permission: AccountInfo<'a>,
    pub payer: AccountInfo<'a>,
    pub system_program: AccountInfo<'a>,
    pub permission_program: AccountInfo<'a>,
    pub flag_byte: u8,
}

impl<'a> CreateEphemeralAtaPermission<'a> {
    #[inline(always)]
    pub fn instruction(&self) -> Instruction {
        Instruction {
            program_id: ESPL_TOKEN_PROGRAM_ID,
            accounts: vec![
                AccountMeta::new(*self.eata.key, false),
                AccountMeta::new(*self.permission.key, false),
                AccountMeta::new(*self.payer.key, true),
                AccountMeta::new_readonly(*self.system_program.key, false),
                AccountMeta::new_readonly(*self.permission_program.key, false),
            ],
            data: vec![
                EphemeralSplDiscriminator::CreateEphemeralAtaPermission as u8,
                self.flag_byte,
            ],
        }
    }

    #[inline(always)]
    pub fn invoke(&self) -> ProgramResult {
        invoke(
            &self.instruction(),
            &[
                self.eata.clone(),
                self.permission.clone(),
                self.payer.clone(),
                self.system_program.clone(),
                self.permission_program.clone(),
            ],
        )
    }

    #[inline(always)]
    pub fn invoke_signed(&self, signers_seeds: &[&[&[u8]]]) -> ProgramResult {
        invoke_signed(
            &self.instruction(),
            &[
                self.eata.clone(),
                self.permission.clone(),
                self.payer.clone(),
                self.system_program.clone(),
                self.permission_program.clone(),
            ],
            signers_seeds,
        )
    }
}
