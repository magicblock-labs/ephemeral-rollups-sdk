use magicblock_magic_program_api::{args::ScheduleTaskArgs, instruction::MagicBlockInstruction};
use solana_program::{instruction::AccountMeta, program::invoke_signed};

use crate::solana_compat::solana::{invoke, AccountInfo, Instruction, ProgramResult};

pub struct ScheduleCrankCpi<'a> {
    pub payer: &'a AccountInfo<'a>,
    pub magic_program: &'a AccountInfo<'a>,
    pub instruction_accounts: &'a [AccountInfo<'a>],
    pub args: ScheduleTaskArgs,
}

impl<'a> ScheduleCrankCpi<'a> {
    pub fn instruction(&self) -> Instruction {
        let mut accounts = vec![self.payer.clone()];
        accounts.extend_from_slice(self.instruction_accounts);

        Instruction::new_with_bincode(
            *self.magic_program.key,
            &MagicBlockInstruction::ScheduleTask(self.args.clone()),
            accounts
                .iter()
                .map(|ai| AccountMeta {
                    pubkey: *ai.key,
                    is_signer: ai.is_signer,
                    is_writable: ai.is_writable,
                })
                .collect(),
        )
    }

    pub fn invoke(&self) -> ProgramResult {
        let mut accounts = Vec::with_capacity(1 + self.instruction_accounts.len());
        accounts.push(self.payer.clone());
        accounts.extend_from_slice(self.instruction_accounts);

        invoke(&self.instruction(), &accounts)
    }

    pub fn invoke_signed(&self, signers_seeds: &[&[&[u8]]]) -> ProgramResult {
        let mut accounts = Vec::with_capacity(1 + self.instruction_accounts.len());
        accounts.push(self.payer.clone());
        accounts.extend_from_slice(self.instruction_accounts);

        invoke_signed(&self.instruction(), &accounts, signers_seeds)
    }
}

pub struct CancelCrankCpi<'a> {
    pub authority: &'a AccountInfo<'a>,
    pub magic_program: &'a AccountInfo<'a>,
    pub crank_id: i64,
}

impl<'a> CancelCrankCpi<'a> {
    pub fn instruction(&self) -> Instruction {
        Instruction::new_with_bincode(
            *self.magic_program.key,
            &MagicBlockInstruction::CancelTask {
                task_id: self.crank_id,
            },
            vec![AccountMeta::new_readonly(*self.authority.key, true)],
        )
    }

    pub fn invoke(&self) -> ProgramResult {
        invoke(&self.instruction(), &[self.authority.clone()])
    }

    pub fn invoke_signed(&self, signers_seeds: &[&[&[u8]]]) -> ProgramResult {
        invoke_signed(
            &self.instruction(),
            &[self.authority.clone()],
            signers_seeds,
        )
    }
}
