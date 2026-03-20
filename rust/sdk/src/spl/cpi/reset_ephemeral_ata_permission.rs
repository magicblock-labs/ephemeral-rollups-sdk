use crate::{
    consts::ESPL_TOKEN_PROGRAM_ID,
    solana_compat::solana::{
        invoke, invoke_signed, AccountInfo, AccountMeta, Instruction, ProgramResult,
    },
    spl::EphemeralSplDiscriminator,
};

/// Build a reset ephemeral ATA permission instruction.
///
/// For details on the flag byte, see the [MemberFlags](`crate::access_control::structs::Member`) struct.
pub struct ResetEphemeralAtaPermission<'a> {
    pub eata: AccountInfo<'a>,
    pub permission: AccountInfo<'a>,
    pub owner: AccountInfo<'a>,
    pub permission_program: AccountInfo<'a>,
    pub flag_byte: u8,
}

impl<'a> ResetEphemeralAtaPermission<'a> {
    #[inline(always)]
    pub fn instruction(&self) -> Instruction {
        Instruction {
            program_id: ESPL_TOKEN_PROGRAM_ID,
            accounts: vec![
                AccountMeta::new_readonly(*self.eata.key, false),
                AccountMeta::new(*self.permission.key, false),
                AccountMeta::new_readonly(*self.owner.key, true),
                AccountMeta::new_readonly(*self.permission_program.key, false),
            ],
            data: vec![
                EphemeralSplDiscriminator::ResetEphemeralAtaPermission as u8,
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
                self.owner.clone(),
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
                self.owner.clone(),
                self.permission_program.clone(),
            ],
            signers_seeds,
        )
    }
}
