use {
    crate::spl::{consts::ESPL_TOKEN_PROGRAM_ID, EphemeralSplDiscriminator},
    core::{mem::MaybeUninit, slice::from_raw_parts},
    pinocchio::{
        cpi::{invoke_signed_with_bounds, Signer},
        instruction::{InstructionAccount, InstructionView},
        AccountView, ProgramResult,
    },
};

/// Create a new ephemeral ATA permission.
pub struct CreateEphemeralAtaPermission<'a> {
    pub eata: &'a AccountView,
    pub permission: &'a AccountView,
    pub payer: &'a AccountView,
    pub system_program: &'a AccountView,
    pub permission_program: &'a AccountView,
    pub eata_bump: u8,
    pub flag_byte: u8,
}

impl<'a> CreateEphemeralAtaPermission<'a> {
    #[inline(always)]
    pub fn invoke(&self) -> ProgramResult {
        self.invoke_signed(&[])
    }

    #[inline(always)]
    pub fn invoke_signed(&self, signers: &[Signer<'_, '_>]) -> ProgramResult {
        let expected_accounts = 5;

        let mut instruction_accounts = [const { MaybeUninit::<InstructionAccount>::uninit() }; 5];
        instruction_accounts[0].write(InstructionAccount::writable(self.eata.address()));
        instruction_accounts[1].write(InstructionAccount::writable(self.permission.address()));
        instruction_accounts[2].write(InstructionAccount::readonly_signer(self.payer.address()));
        instruction_accounts[3].write(InstructionAccount::readonly(self.system_program.address()));
        instruction_accounts[4].write(InstructionAccount::readonly(
            self.permission_program.address(),
        ));

        let mut accounts = [const { MaybeUninit::<&AccountView>::uninit() }; 5];
        accounts[0].write(self.eata);
        accounts[1].write(self.permission);
        accounts[2].write(self.payer);
        accounts[3].write(self.system_program);
        accounts[4].write(self.permission_program);

        let instruction_data = [
            EphemeralSplDiscriminator::CreateEphemeralAtaPermission as u8,
            self.eata_bump,
            self.flag_byte,
        ];

        invoke_signed_with_bounds::<5>(
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
