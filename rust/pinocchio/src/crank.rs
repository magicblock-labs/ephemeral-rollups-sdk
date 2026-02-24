use core::mem::MaybeUninit;

use alloc::vec::Vec;
use magicblock_magic_program_api::{args::ScheduleTaskArgs, instruction::MagicBlockInstruction};
use pinocchio::{
    cpi::{invoke_signed_with_slice, invoke_with_slice, Signer, MAX_CPI_ACCOUNTS},
    error::ProgramError,
    instruction::{InstructionAccount, InstructionView},
    AccountView, ProgramResult,
};

pub struct ScheduleCrankCpi<'a> {
    pub payer: AccountView,
    pub magic_program: AccountView,
    pub instruction_accounts: &'a [&'a AccountView],
    pub args: ScheduleTaskArgs,
}

impl<'a> ScheduleCrankCpi<'a> {
    fn instruction(
        &'a self,
        data: &'a [u8],
    ) -> Result<InstructionView<'a, 'a, 'a, 'a>, ProgramError> {
        let mut accounts =
            [const { MaybeUninit::<InstructionAccount>::uninit() }; MAX_CPI_ACCOUNTS];

        unsafe {
            accounts.get_unchecked_mut(0).write(InstructionAccount {
                address: self.payer.address(),
                is_writable: self.payer.is_writable(),
                is_signer: self.payer.is_signer(),
            });
            for i in 0..self.instruction_accounts.len() {
                accounts.get_unchecked_mut(i + 1).write(InstructionAccount {
                    address: self.instruction_accounts[i].address(),
                    is_writable: self.instruction_accounts[i].is_writable(),
                    is_signer: self.instruction_accounts[i].is_signer(),
                });
            }
        }

        Ok(InstructionView {
            program_id: self.magic_program.address(),
            data: data,
            accounts: unsafe {
                core::slice::from_raw_parts(
                    accounts.as_ptr() as *const InstructionAccount,
                    self.instruction_accounts.len() + 1,
                )
            },
        })
    }

    pub fn invoke(&self) -> ProgramResult {
        let mut accounts = Vec::with_capacity(1 + self.instruction_accounts.len());
        accounts.push(&self.payer);
        accounts.extend_from_slice(self.instruction_accounts);

        let data = bincode::serialize(&MagicBlockInstruction::ScheduleTask(self.args.clone()))
            .map_err(|_| ProgramError::InvalidInstructionData)?;

        invoke_with_slice(&self.instruction(&data)?, &accounts.as_slice())
    }

    pub fn invoke_signed(&self, signers_seeds: &[Signer<'_, '_>]) -> ProgramResult {
        let mut accounts = Vec::with_capacity(1 + self.instruction_accounts.len());
        accounts.push(&self.payer);
        accounts.extend_from_slice(self.instruction_accounts);

        let data = bincode::serialize(&MagicBlockInstruction::ScheduleTask(self.args.clone()))
            .map_err(|_| ProgramError::InvalidInstructionData)?;

        invoke_signed_with_slice(
            &self.instruction(&data)?,
            &accounts.as_slice(),
            signers_seeds,
        )
    }
}
