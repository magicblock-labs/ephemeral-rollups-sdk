use {
    crate::spl::{consts::ESPL_TOKEN_PROGRAM_ID, EphemeralSplDiscriminator},
    core::{mem::MaybeUninit, slice::from_raw_parts},
    pinocchio::{
        cpi::{invoke_signed_with_bounds, Signer},
        instruction::{InstructionAccount, InstructionView},
        AccountView, ProgramResult,
    },
};

/// Deposit SPL tokens into an ephemeral ATA.
pub struct DepositSplTokens<'a> {
    pub authority: &'a AccountView,
    pub eata: &'a AccountView,
    pub vault: &'a AccountView,
    pub mint: &'a AccountView,
    pub user_source_token_acc: &'a AccountView,
    pub vault_token_acc: &'a AccountView,
    pub token_program: &'a AccountView,
    pub amount: u64,
}

impl<'a> DepositSplTokens<'a> {
    #[inline(always)]
    pub fn invoke(&self) -> ProgramResult {
        self.invoke_signed(&[])
    }

    #[inline(always)]
    pub fn invoke_signed(&self, signers: &[Signer<'_, '_>]) -> ProgramResult {
        let expected_accounts = 7;

        let mut instruction_accounts = [const { MaybeUninit::<InstructionAccount>::uninit() }; 7];
        instruction_accounts[0].write(InstructionAccount::writable(self.eata.address()));
        instruction_accounts[1].write(InstructionAccount::readonly(self.vault.address()));
        instruction_accounts[2].write(InstructionAccount::readonly(self.mint.address()));
        instruction_accounts[3].write(InstructionAccount::writable(
            self.user_source_token_acc.address(),
        ));
        instruction_accounts[4].write(InstructionAccount::writable(self.vault_token_acc.address()));
        instruction_accounts[5].write(InstructionAccount::readonly_signer(
            self.authority.address(),
        ));
        instruction_accounts[6].write(InstructionAccount::readonly(self.token_program.address()));

        let mut accounts = [const { MaybeUninit::<&AccountView>::uninit() }; 7];
        accounts[0].write(self.eata);
        accounts[1].write(self.vault);
        accounts[2].write(self.mint);
        accounts[3].write(self.user_source_token_acc);
        accounts[4].write(self.vault_token_acc);
        accounts[5].write(self.authority);
        accounts[6].write(self.token_program);

        let mut instruction_data = [0_u8; 9];
        instruction_data[0] = EphemeralSplDiscriminator::DepositSplTokens as u8;
        instruction_data[1..9].copy_from_slice(&self.amount.to_le_bytes());

        invoke_signed_with_bounds::<7>(
            &InstructionView {
                program_id: &ESPL_TOKEN_PROGRAM_ID,
                accounts: unsafe {
                    from_raw_parts(instruction_accounts.as_ptr() as _, expected_accounts)
                },
                data: unsafe { from_raw_parts(instruction_data.as_ptr() as _, 9) },
            },
            unsafe { from_raw_parts(accounts.as_ptr() as _, expected_accounts) },
            signers,
        )
    }
}
