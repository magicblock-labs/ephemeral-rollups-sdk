use crate::ephem::utils::accounts_to_indices;
use magicblock_core::magic_program::args::{
    ActionArgs, BaseActionArgs, CommitAndUndelegateArgs, CommitTypeArgs, MagicBaseIntentArgs,
    UndelegateTypeArgs,
};
use magicblock_core::magic_program::instruction::MagicBlockInstruction;
use solana_program::account_info::AccountInfo;
use solana_program::entrypoint::ProgramResult;
use solana_program::instruction::{AccountMeta, Instruction};
use solana_program::program::invoke;
use solana_program::pubkey::Pubkey;
use std::collections::HashMap;

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
    pub fn build(&self) -> (Vec<AccountInfo<'info>>, Instruction) {
        // set those to be first
        let mut all_accounts = vec![self.payer.clone(), self.magic_context.clone()];
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
    fn build_args(&self, indices_map: &HashMap<Pubkey, u8>) -> MagicBaseIntentArgs {
        match self {
            MagicAction::BaseActions(call_handlers) => {
                let call_handlers_args = call_handlers
                    .iter()
                    .map(|call_handler| call_handler.to_args(indices_map))
                    .collect();
                MagicBaseIntentArgs::BaseActions(call_handlers_args)
            }
            MagicAction::Commit(value) => MagicBaseIntentArgs::Commit(value.to_args(indices_map)),
            MagicAction::CommitAndUndelegate(value) => {
                MagicBaseIntentArgs::CommitAndUndelegate(value.to_args(indices_map))
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
    pub fn commited_accounts(&self) -> &[AccountInfo<'info>] {
        match self {
            Self::Standalone(commited_accounts) => commited_accounts,
            Self::WithHandler {
                commited_accounts, ..
            } => commited_accounts,
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
                let commited_accounts_indices = accounts_to_indices(commited_accounts, indices_map);
                let call_handlers_args = call_handlers
                    .iter()
                    .map(|call_handler| call_handler.to_args(indices_map))
                    .collect();
                CommitTypeArgs::WithBaseActions {
                    committed_accounts: commited_accounts_indices,
                    base_actions: call_handlers_args,
                }
            }
        }
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

    fn to_args(&self, indices_map: &HashMap<Pubkey, u8>) -> UndelegateTypeArgs {
        match self {
            Self::Standalone => UndelegateTypeArgs::Standalone,
            Self::WithHandler(call_handlers) => {
                let call_handlers_args = call_handlers
                    .iter()
                    .map(|call_handler| call_handler.to_args(indices_map))
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

pub struct CallHandler<'info> {
    pub args: ActionArgs,
    pub compute_units: u32,
    pub escrow_authority: AccountInfo<'info>,
    pub destination_program: AccountInfo<'info>,
    pub accounts: Vec<AccountInfo<'info>>,
}

impl<'info> CallHandler<'info> {
    fn collect_accounts(&self, container: &mut Vec<AccountInfo<'info>>) {
        container.push(self.destination_program.clone());
        container.push(self.escrow_authority.clone());
        container.extend(self.accounts.clone())
    }

    fn to_args(&self, indices_map: &HashMap<Pubkey, u8>) -> BaseActionArgs {
        let destination_program_index = indices_map
            .get(self.destination_program.key)
            .expect(EXPECTED_KEY_MSG);
        let accounts_indices = accounts_to_indices(&self.accounts, indices_map);
        let escrow_authority_index = indices_map
            .get(self.escrow_authority.key)
            .expect(EXPECTED_KEY_MSG);

        BaseActionArgs {
            args: self.args.clone(),
            compute_units: self.compute_units,
            destination_program: *destination_program_index,
            escrow_authority: *escrow_authority_index,
            accounts: accounts_indices,
        }
    }
}

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
    let instruction = if allow_undelegation {
        MagicBlockInstruction::ScheduleCommitAndUndelegate
    } else {
        MagicBlockInstruction::ScheduleCommit
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
    Instruction::new_with_bincode(*magic_program.key, &instruction, account_metas)
}

mod utils {
    use crate::ephem::EXPECTED_KEY_MSG;
    use solana_program::account_info::AccountInfo;
    use solana_program::pubkey::Pubkey;
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
