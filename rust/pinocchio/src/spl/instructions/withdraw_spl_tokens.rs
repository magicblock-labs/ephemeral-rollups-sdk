use {
    crate::spl::{consts::ESPL_TOKEN_PROGRAM_ID, EphemeralSplDiscriminator},
    core::{mem::MaybeUninit, slice::from_raw_parts},
    pinocchio::{
        cpi::{invoke_signed_with_bounds, Signer},
        instruction::{InstructionAccount, InstructionView},
        AccountView, ProgramResult,
    },
};

/// Withdraw SPL tokens from an ephemeral ATA.
pub struct WithdrawSplTokens<'a> {
    pub payer: &'a AccountView,
    pub eata: &'a AccountView,
    pub vault: &'a AccountView,
    pub mint: &'a AccountView,
    pub vault_ata: &'a AccountView,
    pub user_ata: &'a AccountView,
    pub token_program: &'a AccountView,
    pub eata_bump: u8,
    pub amount: u64,
}

impl<'a> WithdrawSplTokens<'a> {
    #[inline(always)]
    pub fn invoke(&self) -> ProgramResult {
        self.invoke_signed(&[])
    }

    #[inline(always)]
    pub fn invoke_signed(&self, signers: &[Signer<'_, '_>]) -> ProgramResult {
        const NUM_ACCOUNTS: usize = 7;

        let mut instruction_accounts =
            [const { MaybeUninit::<InstructionAccount>::uninit() }; NUM_ACCOUNTS];
        instruction_accounts[0].write(InstructionAccount::writable(self.eata.address()));
        instruction_accounts[1].write(InstructionAccount::readonly(self.vault.address()));
        instruction_accounts[2].write(InstructionAccount::readonly(self.mint.address()));
        instruction_accounts[3].write(InstructionAccount::writable(self.vault_ata.address()));
        instruction_accounts[4].write(InstructionAccount::writable(self.user_ata.address()));
        instruction_accounts[5].write(InstructionAccount::readonly_signer(self.payer.address()));
        instruction_accounts[6].write(InstructionAccount::readonly(self.token_program.address()));

        let mut accounts = [const { MaybeUninit::<&AccountView>::uninit() }; NUM_ACCOUNTS];
        accounts[0].write(self.eata);
        accounts[1].write(self.vault);
        accounts[2].write(self.mint);
        accounts[3].write(self.vault_ata);
        accounts[4].write(self.user_ata);
        accounts[5].write(self.payer);
        accounts[6].write(self.token_program);

        let mut instruction_data = [0_u8; 10];
        instruction_data[0] = EphemeralSplDiscriminator::WithdrawSplTokens as u8;
        instruction_data[1..9].copy_from_slice(&self.amount.to_le_bytes());
        instruction_data[9] = self.eata_bump;

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
