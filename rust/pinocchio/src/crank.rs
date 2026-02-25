use core::mem::MaybeUninit;

use alloc::vec::Vec;
use pinocchio::{
    cpi::{invoke_signed_with_slice, invoke_with_slice, Signer, MAX_CPI_ACCOUNTS},
    error::ProgramError,
    instruction::{InstructionAccount, InstructionView},
    AccountView, Address, ProgramResult,
};

pub struct CrankInstruction<'a> {
    pub program_id: Address,
    pub accounts: Vec<InstructionAccount<'a>>,
    pub data: Vec<u8>,
}

impl<'a> CrankInstruction<'a> {
    pub fn serialized_size(&self) -> usize {
        let mut size = 0;
        size += 8; // program_id
        size += 8; // number of accounts
        size += self.accounts.len() * 34; // 32 bytes for address + 1 byte for is_writable + 1 byte for is_signer
        size += 8; // data length
        size += self.data.len();
        size
    }

    pub fn serialize(&self) -> Result<Vec<u8>, ProgramError> {
        let mut data = Vec::with_capacity(self.serialized_size());
        data.extend_from_slice(self.program_id.as_ref());
        data.extend_from_slice((self.accounts.len() as u64).to_le_bytes().as_ref());
        for account in &self.accounts {
            let mut serialized = [0; 34];
            serialized[..32].copy_from_slice(account.address.as_ref());
            serialized[32] = account.is_signer as u8;
            serialized[33] = account.is_writable as u8;
            data.extend_from_slice(&serialized);
        }
        data.extend_from_slice((self.data.len() as u64).to_le_bytes().as_ref());
        data.extend_from_slice(&self.data);
        Ok(data)
    }
}

pub struct ScheduleCrankArgs<'a> {
    pub task_id: i64,
    pub execution_interval_millis: i64,
    pub iterations: i64,
    pub instructions: Vec<CrankInstruction<'a>>,
}

impl<'a> ScheduleCrankArgs<'a> {
    pub fn serialized_size(&self) -> usize {
        let mut size = 0;
        size += 8; // task_id
        size += 8; // execution_interval_millis
        size += 8; // iterations
        size += 8; // number of instructions
        size += self
            .instructions
            .iter()
            .map(|i| i.serialized_size())
            .sum::<usize>();
        size
    }

    pub fn serialize(&self) -> Result<Vec<u8>, ProgramError> {
        let mut data = Vec::with_capacity(self.serialized_size());
        data.extend_from_slice(&self.task_id.to_le_bytes());
        data.extend_from_slice(&self.execution_interval_millis.to_le_bytes());
        data.extend_from_slice(&self.iterations.to_le_bytes());
        data.extend_from_slice((self.instructions.len() as u64).to_le_bytes().as_ref());
        for instruction in &self.instructions {
            data.extend_from_slice(&instruction.serialize()?);
        }
        Ok(data)
    }
}

pub struct ScheduleCrankCpi<'a> {
    pub payer: AccountView,
    pub magic_program: AccountView,
    pub instruction_accounts: &'a [&'a AccountView],
    pub args: ScheduleCrankArgs<'a>,
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
            data,
            accounts: unsafe {
                core::slice::from_raw_parts(
                    accounts.as_ptr() as *const InstructionAccount,
                    self.instruction_accounts.len() + 1,
                )
            },
        })
    }

    fn data(&self) -> Result<Vec<u8>, ProgramError> {
        let mut data = Vec::with_capacity(8 + self.args.serialized_size());
        data.extend_from_slice(6_u32.to_le_bytes().as_ref());
        data.extend_from_slice(&self.args.serialize()?);
        Ok(data)
    }

    pub fn invoke(&self) -> ProgramResult {
        let mut accounts = Vec::with_capacity(1 + self.instruction_accounts.len());
        accounts.push(&self.payer);
        accounts.extend_from_slice(self.instruction_accounts);

        let data = self.data()?;

        invoke_with_slice(&self.instruction(&data)?, accounts.as_slice())
    }

    pub fn invoke_signed(&self, signers_seeds: &[Signer<'_, '_>]) -> ProgramResult {
        let mut accounts = Vec::with_capacity(1 + self.instruction_accounts.len());
        accounts.push(&self.payer);
        accounts.extend_from_slice(self.instruction_accounts);

        let data = self.data()?;

        invoke_signed_with_slice(
            &self.instruction(&data)?,
            accounts.as_slice(),
            signers_seeds,
        )
    }
}

#[cfg(test)]
mod tests {
    use alloc::vec;
    use magicblock_magic_program_api::{
        args::ScheduleTaskArgs, instruction::MagicBlockInstruction,
    };
    use pinocchio::account::RuntimeAccount;
    use solana_instruction::{AccountMeta, Instruction};

    use super::*;

    #[test]
    fn test_schedule_crank_cpi_no_instructions() {
        let this_args = ScheduleCrankArgs {
            task_id: 123,
            execution_interval_millis: 123456,
            iterations: 123456,
            instructions: vec![],
        };
        let api_args = ScheduleTaskArgs {
            task_id: this_args.task_id,
            execution_interval_millis: this_args.execution_interval_millis,
            iterations: this_args.iterations,
            instructions: vec![],
        };

        let this_instruction = ScheduleCrankCpi {
            payer: unsafe {
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
            magic_program: unsafe {
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
            instruction_accounts: &[],
            args: this_args,
        };

        let data = this_instruction.data().unwrap();
        let api_data = bincode::serialize(&MagicBlockInstruction::ScheduleTask(api_args)).unwrap();

        assert_eq!(data, api_data);
    }

    #[test]
    fn test_schedule_crank_cpi_with_instructions() {
        let program_id =
            Address::new_from_array(core::array::from_fn(|i| if i == 0 { 1 } else { 0 }));
        let acc1 = Address::new_from_array(core::array::from_fn(|i| if i == 0 { 2 } else { 0 }));
        let this_args = ScheduleCrankArgs {
            task_id: 123,
            execution_interval_millis: 123,
            iterations: 123,
            instructions: vec![CrankInstruction {
                program_id: program_id.clone(),
                data: vec![1, 2, 3],
                accounts: vec![InstructionAccount {
                    address: &acc1,
                    is_writable: true,
                    is_signer: false,
                }],
            }],
        };
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

        let this_instruction = ScheduleCrankCpi {
            payer: unsafe {
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
            magic_program: unsafe {
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
            instruction_accounts: &[],
            args: this_args,
        };

        let data = this_instruction.data().unwrap();
        let api_data = bincode::serialize(&MagicBlockInstruction::ScheduleTask(api_args)).unwrap();

        assert_eq!(data, api_data);
    }

    #[test]
    fn test_schedule_crank_cpi_with_multiple_instructions() {
        let program_id =
            Address::new_from_array(core::array::from_fn(|i| if i == 0 { 1 } else { 0 }));
        let acc1 = Address::new_from_array(core::array::from_fn(|i| if i == 0 { 2 } else { 0 }));
        let acc2 = Address::new_from_array(core::array::from_fn(|i| if i == 0 { 3 } else { 0 }));
        let this_args = ScheduleCrankArgs {
            task_id: 123,
            execution_interval_millis: 123456,
            iterations: 123456,
            instructions: vec![
                CrankInstruction {
                    program_id: program_id.clone(),
                    data: vec![1, 2, 3],
                    accounts: vec![
                        InstructionAccount {
                            address: &acc1,
                            is_writable: true,
                            is_signer: false,
                        },
                        InstructionAccount {
                            address: &acc2,
                            is_writable: true,
                            is_signer: false,
                        },
                    ],
                },
                CrankInstruction {
                    program_id: program_id.clone(),
                    data: vec![1, 2, 3],
                    accounts: vec![
                        InstructionAccount {
                            address: &acc1,
                            is_writable: true,
                            is_signer: false,
                        },
                        InstructionAccount {
                            address: &acc2,
                            is_writable: true,
                            is_signer: false,
                        },
                    ],
                },
            ],
        };
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

        let this_instruction = ScheduleCrankCpi {
            payer: unsafe {
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
            magic_program: unsafe {
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
            instruction_accounts: &[],
            args: this_args,
        };

        let data = this_instruction.data().unwrap();
        let api_data = bincode::serialize(&MagicBlockInstruction::ScheduleTask(api_args)).unwrap();

        assert_eq!(data, api_data);
    }
}
