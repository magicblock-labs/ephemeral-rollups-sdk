use {
    crate::spl::{consts::ESPL_TOKEN_PROGRAM_ID, EphemeralSplDiscriminator},
    core::{mem::MaybeUninit, slice::from_raw_parts},
    pinocchio::{
        cpi::{invoke_signed_with_bounds, Signer},
        instruction::{InstructionAccount, InstructionView},
        AccountView, ProgramResult,
    },
};

/// Undelegate an ephemeral ATA.
pub struct UndelegateEphemeralAta<'a> {
    pub payer: &'a AccountView,
    pub user_ata: &'a AccountView,
    pub eata: &'a AccountView,
    pub magic_context: &'a AccountView,
    pub magic_program: &'a AccountView,
}

impl<'a> UndelegateEphemeralAta<'a> {
    #[inline(always)]
    pub fn invoke(&self) -> ProgramResult {
        self.invoke_signed(&[])
    }

    #[inline(always)]
    pub fn invoke_signed(&self, signers: &[Signer<'_, '_>]) -> ProgramResult {
        let expected_accounts = 5;

        let mut instruction_accounts = [const { MaybeUninit::<InstructionAccount>::uninit() }; 5];
        instruction_accounts[0].write(InstructionAccount::readonly_signer(self.payer.address()));
        instruction_accounts[1].write(InstructionAccount::writable(self.user_ata.address()));
        instruction_accounts[2].write(InstructionAccount::readonly(self.eata.address()));
        instruction_accounts[3].write(InstructionAccount::writable(self.magic_context.address()));
        instruction_accounts[4].write(InstructionAccount::readonly(self.magic_program.address()));

        let mut accounts = [const { MaybeUninit::<&AccountView>::uninit() }; 5];
        accounts[0].write(self.payer);
        accounts[1].write(self.user_ata);
        accounts[2].write(self.eata);
        accounts[3].write(self.magic_context);
        accounts[4].write(self.magic_program);

        let instruction_data = [EphemeralSplDiscriminator::UndelegateEphemeralAta as u8];

        invoke_signed_with_bounds::<5>(
            &InstructionView {
                program_id: &ESPL_TOKEN_PROGRAM_ID,
                accounts: unsafe {
                    from_raw_parts(instruction_accounts.as_ptr() as _, expected_accounts)
                },
                data: unsafe { from_raw_parts(instruction_data.as_ptr() as _, 1) },
            },
            unsafe { from_raw_parts(accounts.as_ptr() as _, expected_accounts) },
            signers,
        )
    }
}
