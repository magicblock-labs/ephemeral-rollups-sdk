use core::mem::MaybeUninit;

use pinocchio::{
    cpi::{
        invoke_signed_with_bounds, invoke_with_bounds, Signer,
        MAX_CPI_ACCOUNTS as PINOCCHIO_MAX_CPI_ACCOUNTS,
    },
    error::ProgramError,
    instruction::{InstructionAccount, InstructionView},
    AccountView, Address, ProgramResult,
};

const SCHEDULE_CRANK_DISCRIMINANT: [u8; 4] = 6_u32.to_le_bytes();

pub struct CrankInstruction<'a> {
    pub program_id: Address,
    pub accounts: &'a [InstructionAccount<'a>],
    pub data: &'a [u8],
}

impl<'a> CrankInstruction<'a> {
    #[inline(always)]
    pub const fn new(
        program_id: Address,
        accounts: &'a [InstructionAccount<'a>],
        data: &'a [u8],
    ) -> Self {
        Self {
            program_id,
            accounts,
            data,
        }
    }

    pub fn serialized_size(&self) -> usize {
        let mut size = 0;
        size += 32; // program_id
        size += 8; // number of accounts
        size += self.accounts.len() * 34; // 32 bytes for address + 1 byte for is_signer + 1 byte for is_writable
        size += 8; // data length
        size += self.data.len();
        size
    }

    pub fn serialize_into(&self, data: &mut [u8]) -> Result<usize, ProgramError> {
        let required = self.serialized_size();
        if data.len() < required {
            return Err(ProgramError::InvalidInstructionData);
        }

        let mut offset = 0;
        write_serialized_bytes(data, &mut offset, self.program_id.as_ref())?;
        write_serialized_bytes(
            data,
            &mut offset,
            &(self.accounts.len() as u64).to_le_bytes(),
        )?;
        for account in self.accounts {
            write_serialized_bytes(data, &mut offset, account.address.as_ref())?;
            write_serialized_bytes(data, &mut offset, &[account.is_signer as u8])?;
            write_serialized_bytes(data, &mut offset, &[account.is_writable as u8])?;
        }
        write_serialized_bytes(data, &mut offset, &(self.data.len() as u64).to_le_bytes())?;
        write_serialized_bytes(data, &mut offset, self.data)?;

        Ok(offset)
    }
}

pub struct ScheduleCrankArgs<'a> {
    pub task_id: i64,
    pub execution_interval_millis: i64,
    pub iterations: i64,
    pub instructions: &'a [CrankInstruction<'a>],
}

impl<'a> ScheduleCrankArgs<'a> {
    #[inline(always)]
    pub const fn new(task_id: i64, instructions: &'a [CrankInstruction<'a>]) -> Self {
        Self {
            task_id,
            execution_interval_millis: 0,
            iterations: 1,
            instructions,
        }
    }

    #[inline(always)]
    pub const fn execution_interval_millis(mut self, execution_interval_millis: i64) -> Self {
        self.execution_interval_millis = execution_interval_millis;
        self
    }

    #[inline(always)]
    pub const fn iterations(mut self, iterations: i64) -> Self {
        self.iterations = iterations;
        self
    }

    pub fn serialized_size(&self) -> usize {
        let mut size = 0;
        size += 8; // task_id
        size += 8; // execution_interval_millis
        size += 8; // iterations
        size += 8; // number of instructions
        size += self
            .instructions
            .iter()
            .map(|instruction| instruction.serialized_size())
            .sum::<usize>();
        size
    }

    pub fn serialize_into(&self, data: &mut [u8]) -> Result<usize, ProgramError> {
        let required = self.serialized_size();
        if data.len() < required {
            return Err(ProgramError::InvalidInstructionData);
        }

        let mut offset = 0;
        write_serialized_bytes(data, &mut offset, &self.task_id.to_le_bytes())?;
        write_serialized_bytes(
            data,
            &mut offset,
            &self.execution_interval_millis.to_le_bytes(),
        )?;
        write_serialized_bytes(data, &mut offset, &self.iterations.to_le_bytes())?;
        write_serialized_bytes(
            data,
            &mut offset,
            &(self.instructions.len() as u64).to_le_bytes(),
        )?;
        for instruction in self.instructions {
            offset += instruction.serialize_into(&mut data[offset..])?;
        }

        Ok(offset)
    }
}

pub struct ScheduleCrankCpi<'a> {
    pub payer: AccountView,
    pub magic_program: AccountView,
    pub instruction_accounts: &'a [&'a AccountView],
    pub args: ScheduleCrankArgs<'a>,
}

impl<'a> ScheduleCrankCpi<'a> {
    #[inline(always)]
    pub const fn new(
        payer: AccountView,
        magic_program: AccountView,
        instruction_accounts: &'a [&'a AccountView],
        args: ScheduleCrankArgs<'a>,
    ) -> Self {
        Self {
            payer,
            magic_program,
            instruction_accounts,
            args,
        }
    }

    #[inline(always)]
    pub fn builder(payer: AccountView, magic_program: AccountView) -> ScheduleCrankCpiBuilder<'a> {
        ScheduleCrankCpiBuilder::new(payer, magic_program)
    }

    fn instruction<const MAX_ACCOUNTS: usize>(
        &'a self,
        data: &'a [u8],
        accounts: &'a mut [MaybeUninit<InstructionAccount<'a>>; MAX_ACCOUNTS],
    ) -> Result<InstructionView<'a, 'a, 'a, 'a>, ProgramError> {
        let num_accounts = 1 + self.instruction_accounts.len();
        if num_accounts > MAX_ACCOUNTS || MAX_ACCOUNTS > PINOCCHIO_MAX_CPI_ACCOUNTS {
            return Err(ProgramError::InvalidArgument);
        }

        unsafe {
            accounts
                .get_unchecked_mut(0)
                .write(InstructionAccount::writable_signer(self.payer.address()));
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
            data,
            accounts: unsafe {
                core::slice::from_raw_parts(
                    accounts.as_ptr() as *const InstructionAccount,
                    num_accounts,
                )
            },
        })
    }

    pub fn serialized_size(&self) -> usize {
        SCHEDULE_CRANK_DISCRIMINANT.len() + self.args.serialized_size()
    }

    pub fn serialize_into(&self, data: &mut [u8]) -> Result<usize, ProgramError> {
        let required = self.serialized_size();
        if data.len() < required {
            return Err(ProgramError::InvalidInstructionData);
        }

        let mut offset = 0;
        write_serialized_bytes(data, &mut offset, &SCHEDULE_CRANK_DISCRIMINANT)?;
        offset += self.args.serialize_into(&mut data[offset..])?;
        Ok(offset)
    }

    pub fn invoke<const MAX_ACCOUNT_INFOS: usize>(&self, data_buf: &mut [u8]) -> ProgramResult {
        let mut ix_accounts =
            [const { MaybeUninit::<InstructionAccount>::uninit() }; MAX_ACCOUNT_INFOS];
        let num_accounts = 1 + self.instruction_accounts.len();
        if num_accounts > MAX_ACCOUNT_INFOS || MAX_ACCOUNT_INFOS > PINOCCHIO_MAX_CPI_ACCOUNTS {
            return Err(ProgramError::InvalidArgument);
        }

        let mut account_refs = [&self.payer; MAX_ACCOUNT_INFOS];
        account_refs[1..num_accounts].copy_from_slice(self.instruction_accounts);

        let data_len = self.serialize_into(data_buf)?;
        let ix = self.instruction(&data_buf[..data_len], &mut ix_accounts)?;
        Self::do_invoke::<MAX_ACCOUNT_INFOS>(&ix, &account_refs[..num_accounts])
    }

    pub fn invoke_signed<const MAX_ACCOUNT_INFOS: usize>(
        &self,
        data_buf: &mut [u8],
        signers_seeds: &[Signer<'_, '_>],
    ) -> ProgramResult {
        let mut ix_accounts =
            [const { MaybeUninit::<InstructionAccount>::uninit() }; MAX_ACCOUNT_INFOS];
        let num_accounts = 1 + self.instruction_accounts.len();
        if num_accounts > MAX_ACCOUNT_INFOS || MAX_ACCOUNT_INFOS > PINOCCHIO_MAX_CPI_ACCOUNTS {
            return Err(ProgramError::InvalidArgument);
        }

        let mut account_refs = [&self.payer; MAX_ACCOUNT_INFOS];
        account_refs[1..num_accounts].copy_from_slice(self.instruction_accounts);

        let data_len = self.serialize_into(data_buf)?;
        let ix = self.instruction(&data_buf[..data_len], &mut ix_accounts)?;
        Self::do_invoke_signed::<MAX_ACCOUNT_INFOS>(
            &ix,
            &account_refs[..num_accounts],
            signers_seeds,
        )
    }

    #[inline(never)]
    fn do_invoke<const MAX_ACCOUNT_INFOS: usize>(
        ix: &InstructionView,
        account_refs: &[&AccountView],
    ) -> ProgramResult {
        invoke_with_bounds::<MAX_ACCOUNT_INFOS>(ix, account_refs)
    }

    #[inline(never)]
    fn do_invoke_signed<const MAX_ACCOUNT_INFOS: usize>(
        ix: &InstructionView,
        account_refs: &[&AccountView],
        signers_seeds: &[Signer<'_, '_>],
    ) -> ProgramResult {
        invoke_signed_with_bounds::<MAX_ACCOUNT_INFOS>(ix, account_refs, signers_seeds)
    }
}

pub struct ScheduleCrankCpiBuilder<'a> {
    payer: AccountView,
    magic_program: AccountView,
    instruction_accounts: Option<&'a [&'a AccountView]>,
    task_id: Option<i64>,
    execution_interval_millis: i64,
    iterations: i64,
    instructions: Option<&'a [CrankInstruction<'a>]>,
}

impl<'a> ScheduleCrankCpiBuilder<'a> {
    #[inline(always)]
    pub const fn new(payer: AccountView, magic_program: AccountView) -> Self {
        Self {
            payer,
            magic_program,
            instruction_accounts: None,
            task_id: None,
            execution_interval_millis: 0,
            iterations: 1,
            instructions: None,
        }
    }

    #[inline(always)]
    pub const fn instruction_accounts(
        mut self,
        instruction_accounts: &'a [&'a AccountView],
    ) -> Self {
        self.instruction_accounts = Some(instruction_accounts);
        self
    }

    #[inline(always)]
    pub const fn task_id(mut self, task_id: i64) -> Self {
        self.task_id = Some(task_id);
        self
    }

    #[inline(always)]
    pub const fn execution_interval_millis(mut self, execution_interval_millis: i64) -> Self {
        self.execution_interval_millis = execution_interval_millis;
        self
    }

    #[inline(always)]
    pub const fn iterations(mut self, iterations: i64) -> Self {
        self.iterations = iterations;
        self
    }

    #[inline(always)]
    pub const fn instructions(mut self, instructions: &'a [CrankInstruction<'a>]) -> Self {
        self.instructions = Some(instructions);
        self
    }

    pub fn build(self) -> Result<ScheduleCrankCpi<'a>, ProgramError> {
        let Some(task_id) = self.task_id else {
            return Err(ProgramError::InvalidArgument);
        };
        let Some(instruction_accounts) = self.instruction_accounts else {
            return Err(ProgramError::InvalidArgument);
        };
        let Some(instructions) = self.instructions else {
            return Err(ProgramError::InvalidArgument);
        };

        Ok(ScheduleCrankCpi::new(
            self.payer,
            self.magic_program,
            instruction_accounts,
            ScheduleCrankArgs::new(task_id, instructions)
                .execution_interval_millis(self.execution_interval_millis)
                .iterations(self.iterations),
        ))
    }

    #[inline(never)]
    pub fn build_and_invoke<const MAX_ACCOUNT_INFOS: usize>(
        self,
        data_buf: &mut [u8],
    ) -> ProgramResult {
        self.build()?.invoke::<MAX_ACCOUNT_INFOS>(data_buf)
    }

    #[inline(never)]
    pub fn build_and_invoke_signed<const MAX_ACCOUNT_INFOS: usize>(
        self,
        data_buf: &mut [u8],
        signers_seeds: &[Signer<'_, '_>],
    ) -> ProgramResult {
        self.build()?
            .invoke_signed::<MAX_ACCOUNT_INFOS>(data_buf, signers_seeds)
    }
}

pub struct CancelCrankCpi {
    pub authority: AccountView,
    pub task_context: AccountView,
    pub magic_program: AccountView,
    pub crank_id: i64,
}

impl CancelCrankCpi {
    fn instruction<'a>(
        &'a self,
        data: &'a [u8],
        accounts: &'a mut [MaybeUninit<InstructionAccount<'a>>; 2],
    ) -> Result<InstructionView<'a, 'a, 'a, 'a>, ProgramError> {
        unsafe {
            accounts
                .get_unchecked_mut(0)
                .write(if self.authority.is_writable() {
                    InstructionAccount::writable_signer(self.authority.address())
                } else {
                    InstructionAccount::readonly_signer(self.authority.address())
                });
            accounts
                .get_unchecked_mut(1)
                .write(InstructionAccount::writable(self.task_context.address()));
        }

        Ok(InstructionView {
            program_id: self.magic_program.address(),
            data,
            accounts: unsafe {
                core::slice::from_raw_parts(accounts.as_ptr() as *const InstructionAccount, 2)
            },
        })
    }

    fn data(&self) -> [u8; 12] {
        let mut data = [0; 12];
        data[..4].copy_from_slice(&7_u32.to_le_bytes());
        data[4..].copy_from_slice(&self.crank_id.to_le_bytes());
        data
    }

    pub fn invoke(&self) -> ProgramResult {
        let mut ix_accounts = [const { MaybeUninit::<InstructionAccount>::uninit() }; 2];
        let accounts = [&self.authority, &self.task_context];
        let data = self.data();

        invoke_with_bounds::<2>(&self.instruction(&data, &mut ix_accounts)?, &accounts)
    }

    pub fn invoke_signed(&self, signers_seeds: &[Signer<'_, '_>]) -> ProgramResult {
        let mut ix_accounts = [const { MaybeUninit::<InstructionAccount>::uninit() }; 2];
        let accounts = [&self.authority, &self.task_context];
        let data = self.data();

        invoke_signed_with_bounds::<2>(
            &self.instruction(&data, &mut ix_accounts)?,
            &accounts,
            signers_seeds,
        )
    }
}

#[inline(always)]
fn write_serialized_bytes(
    data: &mut [u8],
    offset: &mut usize,
    bytes: &[u8],
) -> Result<(), ProgramError> {
    let end = offset
        .checked_add(bytes.len())
        .ok_or(ProgramError::InvalidInstructionData)?;
    if end > data.len() {
        return Err(ProgramError::InvalidInstructionData);
    }

    data[*offset..end].copy_from_slice(bytes);
    *offset = end;
    Ok(())
}

#[cfg(test)]
mod tests {
    extern crate alloc;

    use alloc::{vec, vec::Vec};
    use magicblock_magic_program_api::{
        args::ScheduleTaskArgs, instruction::MagicBlockInstruction,
    };
    use pinocchio::account::RuntimeAccount;
    use solana_program::instruction::{AccountMeta, Instruction};

    use super::*;

    fn runtime_account(address: Address, is_signer: u8, is_writable: u8) -> RuntimeAccount {
        RuntimeAccount {
            borrow_state: 0,
            is_signer,
            is_writable,
            executable: 0,
            resize_delta: 0,
            address,
            owner: Address::new_from_array([0; 32]),
            lamports: 0,
            data_len: 0,
        }
    }

    fn serialize_schedule_cpi(cpi: &ScheduleCrankCpi<'_>) -> Vec<u8> {
        let mut data = vec![0; cpi.serialized_size()];
        let len = cpi.serialize_into(&mut data).unwrap();
        data.truncate(len);
        data
    }

    #[test]
    fn test_schedule_crank_cpi_no_instructions() {
        let this_args = ScheduleCrankArgs::new(123, &[])
            .execution_interval_millis(123456)
            .iterations(123456);
        let api_args = ScheduleTaskArgs {
            task_id: this_args.task_id,
            execution_interval_millis: this_args.execution_interval_millis,
            iterations: this_args.iterations,
            instructions: vec![],
        };

        let this_instruction = ScheduleCrankCpi::new(
            unsafe {
                AccountView::new_unchecked(&mut RuntimeAccount {
                    borrow_state: 0,
                    is_signer: 0,
                    is_writable: 0,
                    executable: 0,
                    resize_delta: 0,
                    address: Address::new_from_array([0; 32]),
                    owner: Address::new_from_array([0; 32]),
                    lamports: 0,
                    data_len: 0,
                } as *mut RuntimeAccount)
            },
            unsafe {
                AccountView::new_unchecked(&mut RuntimeAccount {
                    borrow_state: 0,
                    is_signer: 0,
                    is_writable: 0,
                    executable: 0,
                    resize_delta: 0,
                    address: Address::new_from_array([0; 32]),
                    owner: Address::new_from_array([0; 32]),
                    lamports: 0,
                    data_len: 0,
                } as *mut RuntimeAccount)
            },
            &[],
            this_args,
        );

        let data = serialize_schedule_cpi(&this_instruction);
        let api_data = bincode1::serialize(&MagicBlockInstruction::ScheduleTask(api_args)).unwrap();

        assert_eq!(data, api_data);
    }

    #[test]
    fn test_schedule_crank_cpi_with_instructions() {
        let program_id =
            Address::new_from_array(core::array::from_fn(|i| if i == 0 { 1 } else { 0 }));
        let acc1 = Address::new_from_array(core::array::from_fn(|i| if i == 0 { 2 } else { 0 }));
        let instruction_accounts = [InstructionAccount::new(&acc1, true, false)];
        let crank_instructions = [CrankInstruction::new(
            program_id.clone(),
            &instruction_accounts,
            &[1, 2, 3],
        )];
        let this_args = ScheduleCrankArgs::new(123, &crank_instructions)
            .execution_interval_millis(123)
            .iterations(123);
        let api_args = ScheduleTaskArgs {
            task_id: this_args.task_id,
            execution_interval_millis: this_args.execution_interval_millis,
            iterations: this_args.iterations,
            instructions: vec![Instruction {
                program_id: program_id.to_bytes().into(),
                accounts: vec![AccountMeta {
                    pubkey: acc1.to_bytes().into(),
                    is_writable: true,
                    is_signer: false,
                }],
                data: vec![1, 2, 3],
            }],
        };

        let this_instruction = ScheduleCrankCpi::new(
            unsafe {
                AccountView::new_unchecked(&mut RuntimeAccount {
                    borrow_state: 0,
                    is_signer: 0,
                    is_writable: 0,
                    executable: 0,
                    resize_delta: 0,
                    address: Address::new_from_array([0; 32]),
                    owner: Address::new_from_array([0; 32]),
                    lamports: 0,
                    data_len: 0,
                } as *mut RuntimeAccount)
            },
            unsafe {
                AccountView::new_unchecked(&mut RuntimeAccount {
                    borrow_state: 0,
                    is_signer: 0,
                    is_writable: 0,
                    executable: 0,
                    resize_delta: 0,
                    address: Address::new_from_array([0; 32]),
                    owner: Address::new_from_array([0; 32]),
                    lamports: 0,
                    data_len: 0,
                } as *mut RuntimeAccount)
            },
            &[],
            this_args,
        );

        let data = serialize_schedule_cpi(&this_instruction);
        let api_data = bincode1::serialize(&MagicBlockInstruction::ScheduleTask(api_args)).unwrap();

        assert_eq!(data, api_data);
    }

    #[test]
    fn test_schedule_crank_cpi_with_multiple_instructions() {
        let program_id =
            Address::new_from_array(core::array::from_fn(|i| if i == 0 { 1 } else { 0 }));
        let acc1 = Address::new_from_array(core::array::from_fn(|i| if i == 0 { 2 } else { 0 }));
        let acc2 = Address::new_from_array(core::array::from_fn(|i| if i == 0 { 3 } else { 0 }));
        let first_accounts = [
            InstructionAccount::new(&acc1, true, false),
            InstructionAccount::new(&acc2, true, false),
        ];
        let second_accounts = [
            InstructionAccount::new(&acc1, true, false),
            InstructionAccount::new(&acc2, true, false),
        ];
        let crank_instructions = [
            CrankInstruction::new(program_id.clone(), &first_accounts, &[1, 2, 3]),
            CrankInstruction::new(program_id.clone(), &second_accounts, &[1, 2, 3]),
        ];
        let this_args = ScheduleCrankArgs::new(123, &crank_instructions)
            .execution_interval_millis(123456)
            .iterations(123456);
        let api_args = ScheduleTaskArgs {
            task_id: this_args.task_id,
            execution_interval_millis: this_args.execution_interval_millis,
            iterations: this_args.iterations,
            instructions: vec![
                Instruction {
                    program_id: program_id.to_bytes().into(),
                    accounts: vec![
                        AccountMeta {
                            pubkey: acc1.to_bytes().into(),
                            is_writable: true,
                            is_signer: false,
                        },
                        AccountMeta {
                            pubkey: acc2.to_bytes().into(),
                            is_writable: true,
                            is_signer: false,
                        },
                    ],
                    data: vec![1, 2, 3],
                },
                Instruction {
                    program_id: program_id.to_bytes().into(),
                    accounts: vec![
                        AccountMeta {
                            pubkey: acc1.to_bytes().into(),
                            is_writable: true,
                            is_signer: false,
                        },
                        AccountMeta {
                            pubkey: acc2.to_bytes().into(),
                            is_writable: true,
                            is_signer: false,
                        },
                    ],
                    data: vec![1, 2, 3],
                },
            ],
        };

        let this_instruction = ScheduleCrankCpi::new(
            unsafe {
                AccountView::new_unchecked(&mut RuntimeAccount {
                    borrow_state: 0,
                    is_signer: 0,
                    is_writable: 0,
                    executable: 0,
                    resize_delta: 0,
                    address: Address::new_from_array([0; 32]),
                    owner: Address::new_from_array([0; 32]),
                    lamports: 0,
                    data_len: 0,
                } as *mut RuntimeAccount)
            },
            unsafe {
                AccountView::new_unchecked(&mut RuntimeAccount {
                    borrow_state: 0,
                    is_signer: 0,
                    is_writable: 0,
                    executable: 0,
                    resize_delta: 0,
                    address: Address::new_from_array([0; 32]),
                    owner: Address::new_from_array([0; 32]),
                    lamports: 0,
                    data_len: 0,
                } as *mut RuntimeAccount)
            },
            &[],
            this_args,
        );

        let data = serialize_schedule_cpi(&this_instruction);
        let api_data = bincode1::serialize(&MagicBlockInstruction::ScheduleTask(api_args)).unwrap();

        assert_eq!(data, api_data);
    }

    #[test]
    fn test_schedule_crank_builder_matches_direct_construction() {
        let mut payer_account = runtime_account(Address::new_from_array([1; 32]), 0, 0);
        let mut magic_program_account = runtime_account(Address::new_from_array([2; 32]), 0, 0);
        let mut task_context_account = runtime_account(Address::new_from_array([3; 32]), 0, 1);
        let payer =
            unsafe { AccountView::new_unchecked(&mut payer_account as *mut RuntimeAccount) };
        let magic_program = unsafe {
            AccountView::new_unchecked(&mut magic_program_account as *mut RuntimeAccount)
        };
        let task_context =
            unsafe { AccountView::new_unchecked(&mut task_context_account as *mut RuntimeAccount) };
        let task_context_accounts = [&task_context];
        let target_program = Address::new_from_array([9; 32]);
        let target_account = Address::new_from_array([8; 32]);
        let execute_accounts = [InstructionAccount::new(&target_account, true, false)];
        let crank_instructions = [CrankInstruction::new(
            target_program,
            &execute_accounts,
            &[7],
        )];

        let direct = ScheduleCrankCpi::new(
            payer.clone(),
            magic_program.clone(),
            &task_context_accounts,
            ScheduleCrankArgs::new(55, &crank_instructions)
                .execution_interval_millis(99)
                .iterations(3),
        );
        let built = ScheduleCrankCpi::builder(payer, magic_program)
            .instruction_accounts(&task_context_accounts)
            .task_id(55)
            .execution_interval_millis(99)
            .iterations(3)
            .instructions(&crank_instructions)
            .build()
            .unwrap();

        assert_eq!(
            serialize_schedule_cpi(&direct),
            serialize_schedule_cpi(&built)
        );
    }

    #[test]
    fn test_schedule_crank_cpi_marks_payer_writable_signer() {
        let mut payer_account = runtime_account(Address::new_from_array([1; 32]), 0, 0);
        let mut magic_program_account = runtime_account(Address::new_from_array([2; 32]), 0, 0);
        let payer =
            unsafe { AccountView::new_unchecked(&mut payer_account as *mut RuntimeAccount) };
        let magic_program = unsafe {
            AccountView::new_unchecked(&mut magic_program_account as *mut RuntimeAccount)
        };
        let instruction = ScheduleCrankCpi::new(
            payer,
            magic_program,
            &[],
            ScheduleCrankArgs::new(1, &[])
                .execution_interval_millis(5)
                .iterations(1),
        );
        let mut data = vec![0; instruction.serialized_size()];
        let data_len = instruction.serialize_into(&mut data).unwrap();
        data.truncate(data_len);
        let mut ix_accounts =
            [const { MaybeUninit::<InstructionAccount>::uninit() }; PINOCCHIO_MAX_CPI_ACCOUNTS];
        let view = instruction.instruction(&data, &mut ix_accounts).unwrap();

        assert!(view.accounts[0].is_writable);
        assert!(view.accounts[0].is_signer);
    }

    #[test]
    fn test_schedule_crank_cpi_instruction_accepts_exact_account_buffer() {
        let mut payer_account = runtime_account(Address::new_from_array([1; 32]), 0, 0);
        let mut magic_program_account = runtime_account(Address::new_from_array([2; 32]), 0, 0);
        let payer =
            unsafe { AccountView::new_unchecked(&mut payer_account as *mut RuntimeAccount) };
        let magic_program = unsafe {
            AccountView::new_unchecked(&mut magic_program_account as *mut RuntimeAccount)
        };
        let instruction = ScheduleCrankCpi::new(
            payer,
            magic_program,
            &[],
            ScheduleCrankArgs::new(1, &[])
                .execution_interval_millis(5)
                .iterations(1),
        );
        let mut data = vec![0; instruction.serialized_size()];
        let data_len = instruction.serialize_into(&mut data).unwrap();
        data.truncate(data_len);
        let mut ix_accounts = [const { MaybeUninit::<InstructionAccount>::uninit() }; 1];
        let view = instruction.instruction(&data, &mut ix_accounts).unwrap();

        assert_eq!(view.accounts.len(), 1);
        assert!(view.accounts[0].is_writable);
        assert!(view.accounts[0].is_signer);
    }

    #[test]
    fn test_cancel_crank_cpi_instruction_data() {
        let mut authority_account = runtime_account(Address::new_from_array([1; 32]), 0, 0);
        let mut task_context_account = runtime_account(Address::new_from_array([2; 32]), 0, 1);
        let mut magic_program_account = runtime_account(Address::new_from_array([3; 32]), 0, 0);
        let authority =
            unsafe { AccountView::new_unchecked(&mut authority_account as *mut RuntimeAccount) };
        let task_context =
            unsafe { AccountView::new_unchecked(&mut task_context_account as *mut RuntimeAccount) };
        let magic_program = unsafe {
            AccountView::new_unchecked(&mut magic_program_account as *mut RuntimeAccount)
        };

        let instruction = CancelCrankCpi {
            authority,
            task_context,
            magic_program,
            crank_id: 42,
        };

        assert_eq!(instruction.data()[..4], 7_u32.to_le_bytes());
        assert_eq!(instruction.data()[4..], 42_i64.to_le_bytes());
    }

    #[test]
    fn test_cancel_crank_cpi_preserves_authority_writability() {
        let mut readonly_authority_account =
            runtime_account(Address::new_from_array([1; 32]), 1, 0);
        let mut writable_authority_account =
            runtime_account(Address::new_from_array([4; 32]), 1, 1);
        let mut task_context_account = runtime_account(Address::new_from_array([2; 32]), 0, 1);
        let mut magic_program_account = runtime_account(Address::new_from_array([3; 32]), 0, 0);

        let readonly_authority = unsafe {
            AccountView::new_unchecked(&mut readonly_authority_account as *mut RuntimeAccount)
        };
        let writable_authority = unsafe {
            AccountView::new_unchecked(&mut writable_authority_account as *mut RuntimeAccount)
        };
        let task_context =
            unsafe { AccountView::new_unchecked(&mut task_context_account as *mut RuntimeAccount) };
        let magic_program = unsafe {
            AccountView::new_unchecked(&mut magic_program_account as *mut RuntimeAccount)
        };

        let readonly_instruction = CancelCrankCpi {
            authority: readonly_authority,
            task_context: task_context.clone(),
            magic_program: magic_program.clone(),
            crank_id: 1,
        };
        let writable_instruction = CancelCrankCpi {
            authority: writable_authority,
            task_context,
            magic_program,
            crank_id: 1,
        };

        let readonly_data = readonly_instruction.data();
        let mut readonly_accounts = [const { MaybeUninit::<InstructionAccount>::uninit() }; 2];
        let readonly_view = readonly_instruction
            .instruction(&readonly_data, &mut readonly_accounts)
            .unwrap();

        assert!(!readonly_view.accounts[0].is_writable);
        assert!(readonly_view.accounts[0].is_signer);

        let writable_data = writable_instruction.data();
        let mut writable_accounts = [const { MaybeUninit::<InstructionAccount>::uninit() }; 2];
        let writable_view = writable_instruction
            .instruction(&writable_data, &mut writable_accounts)
            .unwrap();

        assert!(writable_view.accounts[0].is_writable);
        assert!(writable_view.accounts[0].is_signer);
    }
}
