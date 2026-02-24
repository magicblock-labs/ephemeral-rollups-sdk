use {
    crate::spl::{consts::ESPL_TOKEN_PROGRAM_ID, EphemeralSplDiscriminator},
    core::{mem::MaybeUninit, slice::from_raw_parts},
    pinocchio::{
        cpi::{invoke_signed_with_bounds, Signer},
        instruction::{InstructionAccount, InstructionView},
        AccountView, ProgramResult,
    },
};

/// Delegate an ephemeral ATA permission.
pub struct DelegateEphemeralAtaPermission<'a> {
    pub payer: &'a AccountView,
    pub eata: &'a AccountView,
    pub permission: &'a AccountView,
    pub permission_program: &'a AccountView,
    pub system_program: &'a AccountView,
    pub delegation_buffer: &'a AccountView,
    pub delegation_record: &'a AccountView,
    pub delegation_metadata: &'a AccountView,
    pub delegation_program: &'a AccountView,
    pub validator: &'a AccountView,
    pub eata_bump: u8,
}

impl<'a> DelegateEphemeralAtaPermission<'a> {
    #[inline(always)]
    pub fn invoke(&self) -> ProgramResult {
        self.invoke_signed(&[])
    }

    #[inline(always)]
    pub fn invoke_signed(&self, signers: &[Signer<'_, '_>]) -> ProgramResult {
        const NUM_ACCOUNTS: usize = 10;

        let mut instruction_accounts =
            [const { MaybeUninit::<InstructionAccount>::uninit() }; NUM_ACCOUNTS];
        instruction_accounts[0].write(InstructionAccount::readonly_signer(self.payer.address()));
        instruction_accounts[1].write(InstructionAccount::writable(self.eata.address()));
        instruction_accounts[2].write(InstructionAccount::readonly(
            self.permission_program.address(),
        ));
        instruction_accounts[3].write(InstructionAccount::writable(self.permission.address()));
        instruction_accounts[4].write(InstructionAccount::readonly(self.system_program.address()));
        instruction_accounts[5].write(InstructionAccount::writable(
            self.delegation_buffer.address(),
        ));
        instruction_accounts[6].write(InstructionAccount::writable(
            self.delegation_record.address(),
        ));
        instruction_accounts[7].write(InstructionAccount::writable(
            self.delegation_metadata.address(),
        ));
        instruction_accounts[8].write(InstructionAccount::readonly(
            self.delegation_program.address(),
        ));
        instruction_accounts[9].write(InstructionAccount::readonly(self.validator.address()));

        let mut accounts = [const { MaybeUninit::<&AccountView>::uninit() }; NUM_ACCOUNTS];
        accounts[0].write(self.payer);
        accounts[1].write(self.eata);
        accounts[2].write(self.payer);
        accounts[3].write(self.permission);
        accounts[4].write(self.system_program);
        accounts[5].write(self.delegation_buffer);
        accounts[6].write(self.delegation_record);
        accounts[7].write(self.delegation_metadata);
        accounts[8].write(self.delegation_program);
        accounts[9].write(self.validator);

        let instruction_data = [
            EphemeralSplDiscriminator::DelegateEphemeralAtaPermission as u8,
            self.eata_bump,
        ];

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
