use crate::ephem::utils::accounts_to_indices;
use borsh::{BorshDeserialize, BorshSerialize};
use magicblock_program::magicblock_instruction::{
    CallHandlerArgs, CommitAndUndelegateArgs, CommitTypeArgs, HandlerArgs, MagicActionArgs,
    UndelegateTypeArgs,
};
use solana_program::account_info::AccountInfo;
use solana_program::entrypoint::ProgramResult;
use solana_program::instruction::{AccountMeta, Instruction};
use solana_program::program::invoke;
use solana_program::pubkey::Pubkey;
use std::collections::hash_map::Entry;
use std::collections::HashMap;

const EXPECTED_KEY_MSG: &str = "Key expected to exist!";

/// CPI to trigger a commit for one or more accounts in the ER
#[inline(always)]
pub fn commit_accounts<'a, 'info>(
    payer: &'a AccountInfo<'info>,
    account_infos: Vec<&'a AccountInfo<'info>>,
    magic_context: &'a AccountInfo<'info>,
    magic_program: &'a AccountInfo<'info>,
) -> ProgramResult {
    let ix = create_schedule_commit_ix(payer, &account_infos, magic_context, magic_program, false);
    let mut all_accounts = vec![payer.clone(), magic_context.clone()];
    all_accounts.extend(account_infos.into_iter().cloned());
    invoke(&ix, &all_accounts)
}

/// CPI to trigger a commit and undelegate one or more accounts in the ER
#[inline(always)]
pub fn commit_and_undelegate_accounts<'a, 'info>(
    payer: &'a AccountInfo<'info>,
    account_infos: Vec<&'a AccountInfo<'info>>,
    magic_context: &'a AccountInfo<'info>,
    magic_program: &'a AccountInfo<'info>,
) -> ProgramResult {
    let ix = create_schedule_commit_ix(payer, &account_infos, magic_context, magic_program, true);
    let mut all_accounts = vec![payer.clone(), magic_context.clone()];
    all_accounts.extend(account_infos.into_iter().cloned());
    invoke(&ix, &all_accounts)
}

pub fn create_schedule_commit_ix<'a, 'info>(
    payer: &'a AccountInfo<'info>,
    account_infos: &[&'a AccountInfo<'info>],
    magic_context: &'a AccountInfo<'info>,
    magic_program: &'a AccountInfo<'info>,
    allow_undelegation: bool,
) -> Instruction {
    let instruction_data = if allow_undelegation {
        vec![2, 0, 0, 0]
    } else {
        vec![1, 0, 0, 0]
    };
    let mut account_metas = vec![
        AccountMeta {
            pubkey: *payer.key,
            is_signer: true,
            is_writable: true,
        },
        AccountMeta {
            pubkey: *magic_context.key,
            is_signer: false,
            is_writable: true,
        },
    ];
    account_metas.extend(account_infos.iter().map(|x| AccountMeta {
        pubkey: *x.key,
        is_signer: x.is_signer,
        is_writable: x.is_writable,
    }));
    Instruction::new_with_bytes(*magic_program.key, &instruction_data, account_metas)
}

pub struct InstructionBuilder<'info> {
    magic_context: AccountInfo<'info>,
    magic_program: AccountInfo<'info>,
}

impl<'info> InstructionBuilder<'info> {
    pub fn commit(account_infos: &[AccountInfo<'info>]) -> Self {
        todo!()
    }

    pub fn undelegate() -> Self {
        todo!()
    }
}

struct CallHandler<'info> {
    pub args: HandlerArgs,
    pub destination_program: AccountInfo<'info>,
    pub accounts: Vec<AccountInfo<'info>>,
}

impl<'info> CallHandler<'info> {
    fn collect_accounts(&self, container: &mut Vec<AccountInfo<'info>>) {
        container.push(self.destination_program.clone());
        container.extend(self.accounts.clone())
    }

    fn to_args(&self, indices_map: &HashMap<Pubkey, u8>) -> CallHandlerArgs {
        let destination_program_index = indices_map
            .get(self.destination_program.key)
            .expect(EXPECTED_KEY_MSG);
        let accounts_indices = utils::accounts_to_indices(&self.accounts, indices_map);

        CallHandlerArgs {
            args: self.args.clone(),
            destination_program: *destination_program_index,
            accounts: accounts_indices,
        }
    }
}

pub enum CommitType<'info> {
    /// Regular commit without actions
    Standalone(Vec<AccountInfo<'info>>), // accounts to commit
    /// Commits accounts and runs actions
    WithHandler {
        commited_accounts: Vec<AccountInfo<'info>>,
        call_handlers: Vec<CallHandler<'info>>,
    },
}

impl<'info> CommitType<'info> {
    pub fn commited_accounts(&self) -> &[AccountInfo<'info>] {
        match self {
            Self::Standalone(commited_accounts) => &commited_accounts,
            Self::WithHandler {
                commited_accounts, ..
            } => &commited_accounts,
        }
    }

    fn collect_accounts(&self, accounts_container: &mut Vec<AccountInfo<'info>>) {
        match self {
            Self::Standalone(accounts) => accounts_container.extend(accounts.clone()),
            Self::WithHandler {
                commited_accounts,
                call_handlers,
            } => {
                accounts_container.extend(commited_accounts.clone());
                call_handlers
                    .iter()
                    .for_each(|call_handler| call_handler.collect_accounts(accounts_container));
            }
        }
    }

    fn to_args(&self, indices_map: &HashMap<Pubkey, u8>) -> CommitTypeArgs {
        match self {
            Self::Standalone(accounts) => {
                let accounts_indices = accounts_to_indices(accounts.as_slice(), indices_map);
                CommitTypeArgs::Standalone(accounts_indices)
            }
            Self::WithHandler {
                commited_accounts,
                call_handlers,
            } => {
                let commited_accounts_indices =
                    accounts_to_indices(&commited_accounts, indices_map);
                let call_handlers_args = call_handlers
                    .iter()
                    .map(|call_handler| call_handler.to_args(indices_map))
                    .collect();
                CommitTypeArgs::WithHandler {
                    commited_accounts: commited_accounts_indices,
                    call_handler: call_handlers_args,
                }
            }
        }
    }
}

/// No CommitedAccounts since it is only used with CommitAction.
pub enum UndelegateType<'info> {
    Standalone,
    WithHandler(Vec<CallHandler<'info>>),
}

impl<'info> UndelegateType<'info> {
    fn collect_accounts(&self, accounts_container: &mut Vec<AccountInfo<'info>>) {
        match self {
            Self::Standalone => {}
            Self::WithHandler(call_handlers) => call_handlers
                .iter()
                .for_each(|call_handler| call_handler.collect_accounts(accounts_container)),
        }
    }

    fn to_args(&self, indices_map: &HashMap<Pubkey, u8>) -> UndelegateTypeArgs {
        match self {
            Self::Standalone => UndelegateTypeArgs::Standalone,
            Self::WithHandler(call_handlers) => {
                let call_handlers_args = call_handlers
                    .iter()
                    .map(|call_handler| call_handler.to_args(indices_map))
                    .collect();
                UndelegateTypeArgs::WithHandlder {
                    call_handler: call_handlers_args,
                }
            }
        }
    }
}

pub struct CommitAndUndelegate<'info> {
    commit_type: CommitType<'info>,
    undelegate_type: UndelegateType<'info>,
}

impl<'info> CommitAndUndelegate<'info> {
    fn collect_accounts(&self, accounts_container: &mut Vec<AccountInfo<'info>>) {
        self.commit_type.collect_accounts(accounts_container);
        self.undelegate_type.collect_accounts(accounts_container);
    }

    fn to_args(&self, indices_map: &HashMap<Pubkey, u8>) -> CommitAndUndelegateArgs {
        let commit_type_args = self.commit_type.to_args(indices_map);
        let undelegate_type_args = self.undelegate_type.to_args(indices_map);
        CommitAndUndelegateArgs {
            commit_type: commit_type_args,
            undelegate_type: undelegate_type_args,
        }
    }
}

impl<'info> CommitAndUndelegate<'info> {
    pub fn build(&self) -> Instruction {
        let commited_accounts = self.commit_type.commited_accounts();
        match (&self.commit_type, &self.undelegate_type) {
            (CommitType::Standalone(_), UndelegateType::Standalone) => {}
            (CommitType::WithHandler { .. }, UndelegateType::Standalone) => {}
            (CommitType::Standalone(_), UndelegateType::WithHandler(_)) => {}
            (CommitType::WithHandler { .. }, UndelegateType::WithHandler(_)) => {}
        }

        todo!()
    }
}

pub enum MagicAction<'info> {
    L1Action(Vec<CallHandler<'info>>),
    Commit(CommitType<'info>),
    CommitAndUndelegate(CommitAndUndelegate<'info>),
}

impl<'info> MagicAction<'info> {
    pub fn build(self) -> Instruction {
        todo!()
    }

    /// Collects accounts. May contain duplicates that would have to be processd
    /// TODO: could be &mut Vec<&'a AccountInfo<'info>>
    fn collect_accounts(&self, accounts_container: &mut Vec<AccountInfo<'info>>) {
        match self {
            MagicAction::L1Action(call_handlers) => call_handlers
                .iter()
                .for_each(|call_handler| call_handler.collect_accounts(accounts_container)),
            MagicAction::Commit(commit_type) => commit_type.collect_accounts(accounts_container),
            MagicAction::CommitAndUndelegate(commit_and_undelegate) => {
                commit_and_undelegate.collect_accounts(accounts_container)
            }
        }
    }

    /// Creates argument for CPI
    fn create_args(&self, indices_map: &HashMap<Pubkey, u8>) -> MagicActionArgs {
        match self {
            MagicAction::L1Action(call_handlers) => {
                let call_handlers_args = call_handlers
                    .iter()
                    .map(|call_handler| call_handler.to_args(indices_map))
                    .collect();
                MagicActionArgs::L1Action(call_handlers_args)
            }
            MagicAction::Commit(value) => MagicActionArgs::Commit(value.to_args(indices_map)),
            MagicAction::CommitAndUndelegate(value) => {
                MagicActionArgs::CommitAndUndelegate(value.to_args(indices_map))
            }
        }
    }
}

pub struct MagicInstructionBuilder<'info> {
    pub payer: AccountInfo<'info>,
    pub magic_context: AccountInfo<'info>,
    pub magic_program: AccountInfo<'info>,
    pub magic_action: MagicAction<'info>,
}

impl<'info> MagicInstructionBuilder<'info> {
    pub fn build(&self) -> Instruction {
        // set those to be first
        let mut all_accounts = vec![self.payer.clone(), self.magic_context.clone()];
        self.magic_action.collect_accounts(&mut all_accounts);

        // filter duplicates & get indices map
        let indices_map = self.filter_duplicates_with_map(&mut all_accounts);


        todo!();
    }

    fn filter_duplicates_with_map(
        &self,
        container: &mut Vec<AccountInfo<'info>>,
    ) -> HashMap<Pubkey, u8> {
        let mut map = HashMap::new();
        container.retain(|el| match map.entry(*el.key) {
            Entry::Occupied(_) => false,
            Entry::Vacant(entry) => {
                // insert dummy value. Can't use index counter here
                entry.insert(1);
                true
            }
        });
        // update map with valid indices
        container.iter().enumerate().for_each(|(i, account)| {
            *map.get_mut(account.key).unwrap() = i as u8;
        });

        map
    }
}

mod utils {
    use crate::ephem::EXPECTED_KEY_MSG;
    use solana_program::account_info::AccountInfo;
    use solana_program::pubkey::Pubkey;
    use std::collections::HashMap;

    #[inline(always)]
    pub fn accounts_to_indices<'info>(
        accounts: &[AccountInfo<'info>],
        indices_map: &HashMap<Pubkey, u8>,
    ) -> Vec<u8> {
        accounts
            .iter()
            .map(|account| *indices_map.get(account.key).expect(EXPECTED_KEY_MSG))
            .collect()
    }
}
