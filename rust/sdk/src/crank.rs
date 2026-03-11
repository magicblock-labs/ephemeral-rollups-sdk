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
        let mut accounts = Vec::with_capacity(1 + self.instruction_accounts.len());
        accounts.push(AccountMeta::new(*self.payer.key, true));
        accounts.extend(self.instruction_accounts.iter().map(|ai| AccountMeta {
            pubkey: *ai.key,
            is_signer: ai.is_signer,
            is_writable: ai.is_writable,
        }));

        Instruction::new_with_bincode(
            *self.magic_program.key,
            &MagicBlockInstruction::ScheduleTask(self.args.clone()),
            accounts,
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
    pub task_context: &'a AccountInfo<'a>,
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
            vec![
                if self.authority.is_writable {
                    AccountMeta::new(*self.authority.key, true)
                } else {
                    AccountMeta::new_readonly(*self.authority.key, true)
                },
                AccountMeta::new(*self.task_context.key, false),
            ],
        )
    }

    pub fn invoke(&self) -> ProgramResult {
        let accounts = [self.authority.clone(), self.task_context.clone()];
        invoke(&self.instruction(), &accounts)
    }

    pub fn invoke_signed(&self, signers_seeds: &[&[&[u8]]]) -> ProgramResult {
        let accounts = [self.authority.clone(), self.task_context.clone()];
        invoke_signed(&self.instruction(), &accounts, signers_seeds)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use magicblock_magic_program_api::args::ScheduleTaskArgs;
    use solana_program::{account_info::AccountInfo, clock::Epoch, pubkey::Pubkey};

    fn new_account_info<'a>(
        key: &'a Pubkey,
        is_signer: bool,
        is_writable: bool,
        lamports: &'a mut u64,
        data: &'a mut [u8],
        owner: &'a Pubkey,
    ) -> AccountInfo<'a> {
        AccountInfo::new(
            key,
            is_signer,
            is_writable,
            lamports,
            data,
            owner,
            false,
            Epoch::default(),
        )
    }

    #[test]
    fn schedule_instruction_marks_payer_writable_signer() {
        let payer_key = Pubkey::new_unique();
        let magic_program_key = Pubkey::new_unique();
        let task_context_key = Pubkey::new_unique();
        let owner = Pubkey::new_unique();
        let mut payer_lamports = 0;
        let mut program_lamports = 0;
        let mut task_context_lamports = 0;
        let mut payer_data = [];
        let mut program_data = [];
        let mut task_context_data = [];

        let payer = new_account_info(
            &payer_key,
            false,
            false,
            &mut payer_lamports,
            &mut payer_data,
            &owner,
        );
        let magic_program = new_account_info(
            &magic_program_key,
            false,
            false,
            &mut program_lamports,
            &mut program_data,
            &owner,
        );
        let task_context = new_account_info(
            &task_context_key,
            false,
            true,
            &mut task_context_lamports,
            &mut task_context_data,
            &owner,
        );
        let instruction_accounts = [task_context];

        let instruction = ScheduleCrankCpi {
            payer: &payer,
            magic_program: &magic_program,
            instruction_accounts: &instruction_accounts,
            args: ScheduleTaskArgs {
                task_id: 7,
                execution_interval_millis: 10,
                iterations: 1,
                instructions: vec![],
            },
        }
        .instruction();

        assert_eq!(instruction.accounts[0], AccountMeta::new(payer_key, true));
        assert_eq!(
            instruction.accounts[1],
            AccountMeta::new(task_context_key, false)
        );
    }

    #[test]
    fn cancel_instruction_marks_readonly_authority_signer() {
        let authority_key = Pubkey::new_unique();
        let task_context_key = Pubkey::new_unique();
        let magic_program_key = Pubkey::new_unique();
        let owner = Pubkey::new_unique();
        let mut authority_lamports = 0;
        let mut task_context_lamports = 0;
        let mut program_lamports = 0;
        let mut authority_data = [];
        let mut task_context_data = [];
        let mut program_data = [];

        let authority = new_account_info(
            &authority_key,
            false,
            false,
            &mut authority_lamports,
            &mut authority_data,
            &owner,
        );
        let task_context = new_account_info(
            &task_context_key,
            false,
            false,
            &mut task_context_lamports,
            &mut task_context_data,
            &owner,
        );
        let magic_program = new_account_info(
            &magic_program_key,
            false,
            false,
            &mut program_lamports,
            &mut program_data,
            &owner,
        );

        let instruction = CancelCrankCpi {
            authority: &authority,
            task_context: &task_context,
            magic_program: &magic_program,
            crank_id: 11,
        }
        .instruction();

        assert_eq!(instruction.accounts.len(), 2);
        assert_eq!(
            instruction.accounts[0],
            AccountMeta::new_readonly(authority_key, true)
        );
        assert_eq!(
            instruction.accounts[1],
            AccountMeta::new(task_context_key, false)
        );
    }

    #[test]
    fn cancel_instruction_marks_writable_authority_signer() {
        let authority_key = Pubkey::new_unique();
        let task_context_key = Pubkey::new_unique();
        let magic_program_key = Pubkey::new_unique();
        let owner = Pubkey::new_unique();
        let mut authority_lamports = 0;
        let mut task_context_lamports = 0;
        let mut program_lamports = 0;
        let mut authority_data = [];
        let mut task_context_data = [];
        let mut program_data = [];

        let authority = new_account_info(
            &authority_key,
            false,
            true,
            &mut authority_lamports,
            &mut authority_data,
            &owner,
        );
        let task_context = new_account_info(
            &task_context_key,
            false,
            true,
            &mut task_context_lamports,
            &mut task_context_data,
            &owner,
        );
        let magic_program = new_account_info(
            &magic_program_key,
            false,
            false,
            &mut program_lamports,
            &mut program_data,
            &owner,
        );

        let instruction = CancelCrankCpi {
            authority: &authority,
            task_context: &task_context,
            magic_program: &magic_program,
            crank_id: 11,
        }
        .instruction();

        assert_eq!(instruction.accounts.len(), 2);
        assert_eq!(
            instruction.accounts[0],
            AccountMeta::new(authority_key, true)
        );
        assert_eq!(
            instruction.accounts[1],
            AccountMeta::new(task_context_key, false)
        );
    }
}
