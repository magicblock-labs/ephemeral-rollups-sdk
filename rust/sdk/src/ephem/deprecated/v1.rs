use crate::ephem::deprecated::v1::utils::accounts_to_indices;
use crate::solana_compat::solana::{
    invoke, AccountInfo, AccountMeta, Instruction, ProgramResult, Pubkey,
};
use magicblock_magic_program_api::args::{
    ActionArgs, BaseActionArgs, CommitAndUndelegateArgs, CommitTypeArgs, MagicBaseIntentArgs,
    ShortAccountMeta, UndelegateTypeArgs,
};
use magicblock_magic_program_api::instruction::MagicBlockInstruction;
use std::collections::{HashMap, HashSet};

const EXPECTED_KEY_MSG: &str = "Key expected to exist!";

/// Instruction builder for magicprogram
pub struct MagicInstructionBuilder<'info> {
    pub payer: AccountInfo<'info>,
    pub magic_context: AccountInfo<'info>,
    pub magic_program: AccountInfo<'info>,
    pub magic_action: MagicAction<'info>,
}

impl<'info> MagicInstructionBuilder<'info> {
    /// Build instruction for supplied an action and prepares accounts
    pub fn build(self) -> (Vec<AccountInfo<'info>>, Instruction) {
        // set those to be firstWith
        let mut all_accounts = vec![self.payer, self.magic_context];
        // collect all accounts to be used in instruction
        self.magic_action.collect_accounts(&mut all_accounts);
        // filter duplicates & get indices map
        let indices_map = utils::filter_duplicates_with_map(&mut all_accounts);

        // construct args of ScheduleAction instruction
        let args = self.magic_action.build_args(&indices_map);
        // create accounts metas
        let accounts_meta = all_accounts
            .iter()
            .map(|account| AccountMeta {
                pubkey: *account.key,
                is_signer: account.is_signer,
                is_writable: account.is_writable,
            })
            .collect();

        (
            all_accounts,
            Instruction::new_with_bincode(
                *self.magic_program.key,
                &MagicBlockInstruction::ScheduleBaseIntent(args),
                accounts_meta,
            ),
        )
    }

    /// Builds instruction for action & invokes magicprogram
    pub fn build_and_invoke(self) -> ProgramResult {
        let (accounts, ix) = self.build();
        invoke(&ix, &accounts)
    }
}

/// Action that user wants to perform on base layer
pub enum MagicAction<'info> {
    BaseActions(Vec<CallHandler<'info>>),
    Commit(CommitType<'info>),
    CommitAndUndelegate(CommitAndUndelegate<'info>),
}

impl<'info> MagicAction<'info> {
    /// Collects accounts. May contain duplicates that would have to be processd
    /// TODO: could be &mut Vec<&'a AccountInfo<'info>>
    fn collect_accounts(&self, accounts_container: &mut Vec<AccountInfo<'info>>) {
        match self {
            MagicAction::BaseActions(call_handlers) => call_handlers
                .iter()
                .for_each(|call_handler| call_handler.collect_accounts(accounts_container)),
            MagicAction::Commit(commit_type) => commit_type.collect_accounts(accounts_container),
            MagicAction::CommitAndUndelegate(commit_and_undelegate) => {
                commit_and_undelegate.collect_accounts(accounts_container)
            }
        }
    }

    /// Creates argument for CPI
    fn build_args(self, indices_map: &HashMap<Pubkey, u8>) -> MagicBaseIntentArgs {
        match self {
            MagicAction::BaseActions(call_handlers) => {
                let call_handlers_args = call_handlers
                    .into_iter()
                    .map(|call_handler| call_handler.into_args(indices_map))
                    .collect();
                MagicBaseIntentArgs::BaseActions(call_handlers_args)
            }
            MagicAction::Commit(value) => MagicBaseIntentArgs::Commit(value.into_args(indices_map)),
            MagicAction::CommitAndUndelegate(value) => {
                MagicBaseIntentArgs::CommitAndUndelegate(value.into_args(indices_map))
            }
        }
    }
}

/// Type of commit , can be whether standalone or with some custom actions on Base layer post commit
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
    pub fn committed_accounts(&self) -> &Vec<AccountInfo<'info>> {
        match self {
            Self::Standalone(commited_accounts) => commited_accounts,
            Self::WithHandler {
                commited_accounts, ..
            } => commited_accounts,
        }
    }

    pub(crate) fn committed_accounts_mut(&mut self) -> &mut Vec<AccountInfo<'info>> {
        match self {
            Self::Standalone(commited_accounts) => commited_accounts,
            Self::WithHandler {
                commited_accounts, ..
            } => commited_accounts,
        }
    }

    pub(crate) fn dedup(&mut self) -> HashSet<Pubkey> {
        let committed_accounts = self.committed_accounts_mut();
        let mut seen = HashSet::with_capacity(committed_accounts.len());
        committed_accounts.retain(|el| seen.insert(*el.key));

        seen
    }

    pub(crate) fn collect_accounts(&self, accounts_container: &mut Vec<AccountInfo<'info>>) {
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

    pub(crate) fn into_args(self, indices_map: &HashMap<Pubkey, u8>) -> CommitTypeArgs {
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
                    accounts_to_indices(commited_accounts.as_slice(), indices_map);
                let call_handlers_args = call_handlers
                    .into_iter()
                    .map(|call_handler| call_handler.into_args(indices_map))
                    .collect();
                CommitTypeArgs::WithBaseActions {
                    committed_accounts: commited_accounts_indices,
                    base_actions: call_handlers_args,
                }
            }
        }
    }

    pub(crate) fn merge(&mut self, mut other: Self) {
        let take = |value: &mut _| -> (Vec<AccountInfo>, Vec<CallHandler>) {
            use std::mem::take;

            match value {
                CommitType::Standalone(value) => (take(value), vec![]),
                CommitType::WithHandler {
                    commited_accounts,
                    call_handlers,
                } => (take(commited_accounts), take(call_handlers)),
            }
        };

        let (mut accounts, mut actions) = take(self);
        let (other1, other2) = take(&mut other);
        accounts.extend(other1);
        actions.extend(other2);

        if actions.is_empty() {
            *self = CommitType::Standalone(accounts)
        } else {
            *self = CommitType::WithHandler {
                commited_accounts: accounts,
                call_handlers: actions,
            }
        };
    }
}

/// Type of undelegate, can be whether standalone or with some custom actions on Base layer post commit
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

    fn into_args(self, indices_map: &HashMap<Pubkey, u8>) -> UndelegateTypeArgs {
        match self {
            Self::Standalone => UndelegateTypeArgs::Standalone,
            Self::WithHandler(call_handlers) => {
                let call_handlers_args = call_handlers
                    .into_iter()
                    .map(|call_handler| call_handler.into_args(indices_map))
                    .collect();
                UndelegateTypeArgs::WithBaseActions {
                    base_actions: call_handlers_args,
                }
            }
        }
    }
}

pub struct CommitAndUndelegate<'info> {
    pub commit_type: CommitType<'info>,
    pub undelegate_type: UndelegateType<'info>,
}

impl<'info> CommitAndUndelegate<'info> {
    pub(crate) fn collect_accounts(&self, accounts_container: &mut Vec<AccountInfo<'info>>) {
        self.commit_type.collect_accounts(accounts_container);
        self.undelegate_type.collect_accounts(accounts_container);
    }

    pub(crate) fn into_args(self, indices_map: &HashMap<Pubkey, u8>) -> CommitAndUndelegateArgs {
        let commit_type_args = self.commit_type.into_args(indices_map);
        let undelegate_type_args = self.undelegate_type.into_args(indices_map);
        CommitAndUndelegateArgs {
            commit_type: commit_type_args,
            undelegate_type: undelegate_type_args,
        }
    }

    pub(crate) fn dedup(&mut self) -> HashSet<Pubkey> {
        self.commit_type.dedup()
    }

    pub(crate) fn merge(&mut self, other: Self) {
        self.commit_type.merge(other.commit_type);

        let this = std::mem::replace(&mut self.undelegate_type, UndelegateType::Standalone);
        self.undelegate_type = match (this, other.undelegate_type) {
            (UndelegateType::Standalone, UndelegateType::Standalone) => UndelegateType::Standalone,
            (UndelegateType::Standalone, UndelegateType::WithHandler(v))
            | (UndelegateType::WithHandler(v), UndelegateType::Standalone) => {
                UndelegateType::WithHandler(v)
            }
            (UndelegateType::WithHandler(mut a), UndelegateType::WithHandler(b)) => {
                a.extend(b);
                UndelegateType::WithHandler(a)
            }
        };
    }
}

pub struct CallHandler<'info> {
    pub args: ActionArgs,
    pub compute_units: u32,
    pub escrow_authority: AccountInfo<'info>,
    pub destination_program: Pubkey,
    pub accounts: Vec<ShortAccountMeta>,
}

impl<'info> CallHandler<'info> {
    pub(crate) fn collect_accounts(&self, container: &mut Vec<AccountInfo<'info>>) {
        container.push(self.escrow_authority.clone());
    }

    pub(crate) fn into_args(self, indices_map: &HashMap<Pubkey, u8>) -> BaseActionArgs {
        let escrow_authority_index = indices_map
            .get(self.escrow_authority.key)
            .expect(EXPECTED_KEY_MSG);

        BaseActionArgs {
            args: self.args,
            compute_units: self.compute_units,
            destination_program: self.destination_program.to_bytes().into(),
            escrow_authority: *escrow_authority_index,
            accounts: self.accounts,
        }
    }
}

pub(crate) mod utils {
    use super::EXPECTED_KEY_MSG;
    use crate::solana_compat::solana::{AccountInfo, Pubkey};
    use std::collections::hash_map::Entry;
    use std::collections::HashMap;

    #[inline(always)]
    pub fn accounts_to_indices(
        accounts: &[AccountInfo],
        indices_map: &HashMap<Pubkey, u8>,
    ) -> Vec<u8> {
        accounts
            .iter()
            .map(|account| *indices_map.get(account.key).expect(EXPECTED_KEY_MSG))
            .collect()
    }

    /// Removes duplicates from array by pubkey
    /// Returns a map of key to index in cleaned array
    pub fn filter_duplicates_with_map(container: &mut Vec<AccountInfo>) -> HashMap<Pubkey, u8> {
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

#[test]
fn test_instruction_equality() {
    let serialized = bincode::serialize(&MagicBlockInstruction::ScheduleCommit).unwrap();
    assert_eq!(vec![1, 0, 0, 0], serialized);

    let serialized =
        bincode::serialize(&MagicBlockInstruction::ScheduleCommitAndUndelegate).unwrap();
    assert_eq!(vec![2, 0, 0, 0], serialized);
}
