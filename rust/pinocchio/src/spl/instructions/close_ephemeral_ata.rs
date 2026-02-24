use {
    crate::spl::{consts::ESPL_TOKEN_PROGRAM_ID, EphemeralSplDiscriminator},
    core::{mem::MaybeUninit, slice::from_raw_parts},
    pinocchio::{
        cpi::{invoke_signed_with_bounds, Signer},
        instruction::{InstructionAccount, InstructionView},
        AccountView, ProgramResult,
    },
};

/// Close an ephemeral ATA.
pub struct CloseEphemeralAta<'a> {
    pub payer: &'a AccountView,
    pub eata: &'a AccountView,
    pub user: &'a AccountView,
}

impl<'a> CloseEphemeralAta<'a> {
    #[inline(always)]
    pub fn invoke(&self) -> ProgramResult {
        self.invoke_signed(&[])
    }

    #[inline(always)]
    pub fn invoke_signed(&self, signers: &[Signer<'_, '_>]) -> ProgramResult {
        const NUM_ACCOUNTS: usize = 3;
        let mut instruction_accounts =
            [const { MaybeUninit::<InstructionAccount>::uninit() }; NUM_ACCOUNTS];
        instruction_accounts[0].write(InstructionAccount::readonly(self.user.address()));
        instruction_accounts[1].write(InstructionAccount::writable(self.eata.address()));
        instruction_accounts[2].write(InstructionAccount::writable_signer(self.payer.address()));

        let mut accounts = [const { MaybeUninit::<&AccountView>::uninit() }; NUM_ACCOUNTS];
        accounts[0].write(self.user);
        accounts[1].write(self.eata);
        accounts[2].write(self.payer);

        let instruction_data = [EphemeralSplDiscriminator::CloseEphemeralAta as u8];

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
