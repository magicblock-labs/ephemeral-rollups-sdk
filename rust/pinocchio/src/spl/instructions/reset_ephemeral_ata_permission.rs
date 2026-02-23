use {
    crate::spl::{consts::ESPL_TOKEN_PROGRAM_ID, EphemeralSplDiscriminator},
    core::{mem::MaybeUninit, slice::from_raw_parts},
    pinocchio::{
        cpi::{invoke_signed_with_bounds, Signer},
        instruction::{InstructionAccount, InstructionView},
        AccountView, ProgramResult,
    },
};

/// Reset an ephemeral ATA permission.
///
/// For details on the flag byte, see [MemberFlags](`crate::acl::types::MemberFlags`).
pub struct ResetEphemeralAtaPermission<'a> {
    pub eata: &'a AccountView,
    pub permission: &'a AccountView,
    pub owner: &'a AccountView,
    pub permission_program: &'a AccountView,
    pub bump: u8,
    pub flag_byte: u8,
}

impl<'a> ResetEphemeralAtaPermission<'a> {
    #[inline(always)]
    pub fn invoke(&self) -> ProgramResult {
        self.invoke_signed(&[])
    }

    #[inline(always)]
    pub fn invoke_signed(&self, signers: &[Signer<'_, '_>]) -> ProgramResult {
        let expected_accounts = 4;

        let mut instruction_accounts = [const { MaybeUninit::<InstructionAccount>::uninit() }; 4];
        instruction_accounts[0].write(InstructionAccount::writable(self.eata.address()));
        instruction_accounts[1].write(InstructionAccount::writable(self.permission.address()));
        instruction_accounts[2].write(InstructionAccount::readonly_signer(self.owner.address()));
        instruction_accounts[3].write(InstructionAccount::readonly(
            self.permission_program.address(),
        ));

        let mut accounts = [const { MaybeUninit::<&AccountView>::uninit() }; 4];
        accounts[0].write(self.eata);
        accounts[1].write(self.permission);
        accounts[2].write(self.owner);
        accounts[3].write(self.permission_program);

        let instruction_data = [
            EphemeralSplDiscriminator::ResetEphemeralAtaPermission as u8,
            self.bump,
            self.flag_byte,
        ];

        invoke_signed_with_bounds::<4>(
            &InstructionView {
                program_id: &ESPL_TOKEN_PROGRAM_ID,
                accounts: unsafe {
                    from_raw_parts(instruction_accounts.as_ptr() as _, expected_accounts)
                },
                data: unsafe { from_raw_parts(instruction_data.as_ptr() as _, 3) },
            },
            unsafe { from_raw_parts(accounts.as_ptr() as _, expected_accounts) },
            signers,
        )
    }
}
