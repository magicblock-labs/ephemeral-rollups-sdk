use {
    crate::spl::{consts::ESPL_TOKEN_PROGRAM_ID, EphemeralSplDiscriminator},
    core::{mem::MaybeUninit, slice::from_raw_parts},
    pinocchio::{
        cpi::{invoke_signed_with_bounds, Signer},
        instruction::{InstructionAccount, InstructionView},
        AccountView, ProgramResult,
    },
};

/// Initialize a global vault for a mint.
pub struct InitializeGlobalVault<'a> {
    pub payer: &'a AccountView,
    pub vault: &'a AccountView,
    pub mint: &'a AccountView,
    pub vault_ephemeral_ata: &'a AccountView,
    pub vault_ata: &'a AccountView,
    pub token_program: &'a AccountView,
    pub associated_token_program: &'a AccountView,
    pub system_program: &'a AccountView,
}

impl<'a> InitializeGlobalVault<'a> {
    #[inline(always)]
    pub fn invoke(&self) -> ProgramResult {
        self.invoke_signed(&[])
    }

    #[inline(always)]
    pub fn invoke_signed(&self, signers: &[Signer<'_, '_>]) -> ProgramResult {
        const NUM_ACCOUNTS: usize = 8;

        let mut instruction_accounts =
            [const { MaybeUninit::<InstructionAccount>::uninit() }; NUM_ACCOUNTS];
        instruction_accounts[0].write(InstructionAccount::writable(self.vault.address()));
        instruction_accounts[1].write(InstructionAccount::writable_signer(self.payer.address()));
        instruction_accounts[2].write(InstructionAccount::readonly(self.mint.address()));
        instruction_accounts[3].write(InstructionAccount::writable(
            self.vault_ephemeral_ata.address(),
        ));
        instruction_accounts[4].write(InstructionAccount::writable(self.vault_ata.address()));
        instruction_accounts[5].write(InstructionAccount::readonly(self.token_program.address()));
        instruction_accounts[6].write(InstructionAccount::readonly(
            self.associated_token_program.address(),
        ));
        instruction_accounts[7].write(InstructionAccount::readonly(self.system_program.address()));

        let mut accounts = [const { MaybeUninit::<&AccountView>::uninit() }; NUM_ACCOUNTS];
        accounts[0].write(self.vault);
        accounts[1].write(self.payer);
        accounts[2].write(self.mint);
        accounts[3].write(self.vault_ephemeral_ata);
        accounts[4].write(self.vault_ata);
        accounts[5].write(self.token_program);
        accounts[6].write(self.associated_token_program);
        accounts[7].write(self.system_program);

        let instruction_data = [EphemeralSplDiscriminator::InitializeGlobalVault as u8];

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
