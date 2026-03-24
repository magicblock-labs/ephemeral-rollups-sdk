use {
    crate::spl::{consts::ESPL_TOKEN_PROGRAM_ID, EphemeralSplDiscriminator},
    core::{mem::MaybeUninit, slice::from_raw_parts},
    pinocchio::{
        cpi::{invoke_signed_with_bounds, Signer},
        instruction::{InstructionAccount, InstructionView},
        AccountView, Address, ProgramResult,
    },
};

/// Delegate an ephemeral ATA.
pub struct DelegateEphemeralAta<'a> {
    pub payer: &'a AccountView,
    pub eata: &'a AccountView,
    pub espl_token_program: &'a AccountView,
    pub delegation_buffer: &'a AccountView,
    pub delegation_record: &'a AccountView,
    pub delegation_metadata: &'a AccountView,
    pub delegation_program: &'a AccountView,
    pub system_program: &'a AccountView,
    pub validator: Option<Address>,
}

impl<'a> DelegateEphemeralAta<'a> {
    #[inline(always)]
    pub fn invoke(&self) -> ProgramResult {
        self.invoke_signed(&[])
    }

    #[inline(always)]
    pub fn invoke_signed(&self, signers: &[Signer<'_, '_>]) -> ProgramResult {
        const NUM_ACCOUNTS: usize = 8;

        let mut instruction_accounts =
            [const { MaybeUninit::<InstructionAccount>::uninit() }; NUM_ACCOUNTS];
        instruction_accounts[0].write(InstructionAccount::readonly_signer(self.payer.address()));
        instruction_accounts[1].write(InstructionAccount::writable(self.eata.address()));
        instruction_accounts[2].write(InstructionAccount::readonly(
            self.espl_token_program.address(),
        ));
        instruction_accounts[3].write(InstructionAccount::writable(
            self.delegation_buffer.address(),
        ));
        instruction_accounts[4].write(InstructionAccount::writable(
            self.delegation_record.address(),
        ));
        instruction_accounts[5].write(InstructionAccount::writable(
            self.delegation_metadata.address(),
        ));
        instruction_accounts[6].write(InstructionAccount::readonly(
            self.delegation_program.address(),
        ));
        instruction_accounts[7].write(InstructionAccount::readonly(self.system_program.address()));

        let mut accounts = [const { MaybeUninit::<&AccountView>::uninit() }; NUM_ACCOUNTS];
        accounts[0].write(self.payer);
        accounts[1].write(self.eata);
        accounts[2].write(self.espl_token_program);
        accounts[3].write(self.delegation_buffer);
        accounts[4].write(self.delegation_record);
        accounts[5].write(self.delegation_metadata);
        accounts[6].write(self.delegation_program);
        accounts[7].write(self.system_program);

        let mut instruction_data = [0_u8; 33];
        instruction_data[0] = EphemeralSplDiscriminator::DelegateEphemeralAta as u8;
        let instruction_data_len = if let Some(validator) = &self.validator {
            instruction_data[1..33].copy_from_slice(validator.as_ref());
            33
        } else {
            1
        };

        invoke_signed_with_bounds::<NUM_ACCOUNTS>(
            &InstructionView {
                program_id: &ESPL_TOKEN_PROGRAM_ID,
                accounts: unsafe {
                    from_raw_parts(instruction_accounts.as_ptr() as _, NUM_ACCOUNTS)
                },
                data: &instruction_data[..instruction_data_len],
            },
            unsafe { from_raw_parts(accounts.as_ptr() as _, NUM_ACCOUNTS) },
            signers,
        )
    }
}
