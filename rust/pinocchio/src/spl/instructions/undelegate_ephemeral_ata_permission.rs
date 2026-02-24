use {
    crate::spl::{consts::ESPL_TOKEN_PROGRAM_ID, EphemeralSplDiscriminator},
    core::{mem::MaybeUninit, slice::from_raw_parts},
    pinocchio::{
        cpi::{invoke_signed_with_bounds, Signer},
        instruction::{InstructionAccount, InstructionView},
        AccountView, ProgramResult,
    },
};

/// Undelegate an ephemeral ATA permission.
pub struct UndelegateEphemeralAtaPermission<'a> {
    pub payer: &'a AccountView,
    pub eata: &'a AccountView,
    pub permission: &'a AccountView,
    pub permission_program: &'a AccountView,
    pub magic_program: &'a AccountView,
    pub magic_context: &'a AccountView,
}

impl<'a> UndelegateEphemeralAtaPermission<'a> {
    #[inline(always)]
    pub fn invoke(&self) -> ProgramResult {
        self.invoke_signed(&[])
    }

    #[inline(always)]
    pub fn invoke_signed(&self, signers: &[Signer<'_, '_>]) -> ProgramResult {
        const NUM_ACCOUNTS: usize = 6;

        let mut instruction_accounts =
            [const { MaybeUninit::<InstructionAccount>::uninit() }; NUM_ACCOUNTS];
        instruction_accounts[0].write(InstructionAccount::readonly_signer(self.payer.address()));
        instruction_accounts[1].write(InstructionAccount::readonly(self.eata.address()));
        instruction_accounts[2].write(InstructionAccount::writable(self.permission.address()));
        instruction_accounts[3].write(InstructionAccount::readonly(
            self.permission_program.address(),
        ));
        instruction_accounts[4].write(InstructionAccount::readonly(self.magic_program.address()));
        instruction_accounts[5].write(InstructionAccount::writable(self.magic_context.address()));

        let mut accounts = [const { MaybeUninit::<&AccountView>::uninit() }; NUM_ACCOUNTS];
        accounts[0].write(self.payer);
        accounts[1].write(self.eata);
        accounts[2].write(self.permission);
        accounts[3].write(self.permission_program);
        accounts[4].write(self.magic_program);
        accounts[5].write(self.magic_context);

        let instruction_data = [EphemeralSplDiscriminator::UndelegateEphemeralAtaPermission as u8];

        invoke_signed_with_bounds::<NUM_ACCOUNTS>(
            &InstructionView {
                program_id: &ESPL_TOKEN_PROGRAM_ID,
                accounts: unsafe {
                    from_raw_parts(instruction_accounts.as_ptr() as _, NUM_ACCOUNTS)
                },
                data: &instruction_data,
            },
            unsafe { from_raw_parts(accounts.as_ptr() as _, NUM_ACCOUNTS) },
            signers,
        )
    }
}
