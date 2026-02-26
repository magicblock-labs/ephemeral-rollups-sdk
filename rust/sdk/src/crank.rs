use magicblock_magic_program_api::{args::ScheduleTaskArgs, instruction::MagicBlockInstruction};

use crate::solana_compat::solana::{
    invoke, invoke_signed, AccountInfo, AccountMeta, Instruction, ProgramResult,
};

pub struct ScheduleCrankCpi<'a> {
    pub payer: &'a AccountInfo<'a>,
    pub magic_program: &'a AccountInfo<'a>,
    pub instruction_accounts: &'a [AccountInfo<'a>],
    pub args: ScheduleTaskArgs,
}

impl<'a> ScheduleCrankCpi<'a> {
    pub fn instruction(&self) -> Instruction {
        let accounts = Self::build_accounts(self.payer, self.instruction_accounts);

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
        let accounts = Self::build_accounts(self.payer, self.instruction_accounts);

        invoke(&self.instruction(), &accounts)
    }

    pub fn invoke_signed(&self, signers_seeds: &[&[&[u8]]]) -> ProgramResult {
        let accounts = Self::build_accounts(self.payer, self.instruction_accounts);

        invoke_signed(&self.instruction(), &accounts, signers_seeds)
    }

    fn build_accounts(
        payer: &'a AccountInfo<'a>,
        instruction_accounts: &'a [AccountInfo<'a>],
    ) -> Vec<AccountInfo<'a>> {
        let mut accounts = Vec::with_capacity(1 + instruction_accounts.len());
        accounts.push(payer.clone());
        accounts.extend_from_slice(instruction_accounts);
        accounts
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
        invoke(&self.instruction(), core::slice::from_ref(self.authority))
    }

    pub fn invoke_signed(&self, signers_seeds: &[&[&[u8]]]) -> ProgramResult {
        invoke_signed(
            &self.instruction(),
            core::slice::from_ref(self.authority),
            signers_seeds,
        )
    }
}
