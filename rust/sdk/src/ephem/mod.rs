pub use crate::ephem::deprecated::v0::{
    commit_accounts, commit_and_undelegate_accounts, create_schedule_commit_ix,
};
use crate::ephem::deprecated::v1::{
    CallHandler, CommitAndUndelegate, CommitType, MagicAction, UndelegateType,
};
use crate::solana_compat::solana::{invoke, AccountInfo, Instruction, ProgramResult};
use std::cmp::Ordering;
use std::collections::VecDeque;

pub mod deprecated;

/// Describes types of Base Intents
pub type MagicBaseIntent<'info> = MagicAction<'info>;

pub struct MagicIntentBundleBuilder<'info> {
    payer: AccountInfo<'info>,
    magic_context: AccountInfo<'info>,
    magic_program: AccountInfo<'info>,
    intent_bundle: MagicIntentBundle<'info>,
}

impl<'info> MagicIntentBundleBuilder<'info> {
    pub fn new(
        payer: AccountInfo<'info>,
        magic_context: AccountInfo<'info>,
        magic_program: AccountInfo<'info>,
    ) -> Self {
        Self {
            payer,
            magic_context,
            magic_program,
            intent_bundle: MagicIntentBundle::default(),
        }
    }

    pub fn add_intent(mut self, intent: MagicBaseIntent<'info>) -> Self {
        self.intent_bundle.add_intent(intent);
        self
    }

    pub fn add_commit_intent(mut self, commit: CommitType<'info>) -> Self {
        self.intent_bundle.add_intent(MagicAction::Commit(commit));
        self
    }

    pub fn add_commit_and_undelegate_intent(mut self, value: CommitAndUndelegate<'info>) -> Self {
        self.intent_bundle
            .add_intent(MagicAction::CommitAndUndelegate(value));
        self
    }

    pub fn add_base_actions_intent(
        mut self,
        actions: impl IntoIterator<Item = CallHandler<'info>>,
    ) -> Self {
        self.intent_bundle
            .add_intent(MagicAction::BaseActions(actions.into_iter().collect()));
        self
    }

    pub fn build(self) -> (Vec<AccountInfo<'info>>, Instruction) {
        todo!()
    }

    pub fn build_and_invoke(self) -> ProgramResult {
        let (accounts, ix) = self.build();
        invoke(&ix, &accounts)
    }
}

/// Bundle of Intents
///
/// Note: if `CommitIntent` & `CommitAndUndelegateIntent` has an account overlap
/// they will be undelegated
///
/// Intents assumed to be independent and self-sufficient,
/// hence order in which they were inserted doesn't matter
#[derive(Default)]
struct MagicIntentBundle<'info> {
    standalone_actions: Vec<CallHandler<'info>>,
    commit_intent: Option<CommitType<'info>>,
    commit_and_undelegate_intent: Option<CommitAndUndelegate<'info>>,
}

impl<'info> MagicIntentBundle<'info> {
    fn add_intent(&mut self, intent: MagicBaseIntent<'info>) {
        match intent {
            MagicBaseIntent::BaseActions(value) => self.standalone_actions.extend(value),
            MagicBaseIntent::Commit(value) => {
                if let Some(ref mut commit_accounts) = self.commit_intent {
                    commit_accounts.merge(value);
                } else {
                    self.commit_intent = Some(value);
                }
            }
            MagicBaseIntent::CommitAndUndelegate(value) => {
                if let Some(ref mut commit_and_undelegate) = self.commit_and_undelegate_intent {
                    commit_and_undelegate.merge(value);
                } else {
                    self.commit_and_undelegate_intent = Some(value);
                }
            }
        }
    }
}


/// Builder of Commit Intent
pub struct CommitIntentBuilder<'a, 'info> {
    accounts: &'a [AccountInfo<'info>],
    actions: Vec<CallHandler<'info>>,
}

impl<'a, 'info> CommitIntentBuilder<'a, 'info> {
    pub fn new(accounts: &'a [AccountInfo<'info>]) -> Self {
        Self {
            accounts,
            actions: vec![],
        }
    }

    pub fn add_post_commit_actions(
        mut self,
        actions: impl IntoIterator<Item = CallHandler<'info>>,
    ) -> CommitIntentBuilder<'a, 'info> {
        self.actions.extend(actions);
        self
    }

    /// Builds and returns Commit Intent Type
    pub fn build(self) -> CommitType<'info> {
        let commited_accounts = self.accounts.to_vec();
        if self.actions.is_empty() {
            CommitType::Standalone(commited_accounts)
        } else {
            CommitType::WithHandler {
                commited_accounts,
                call_handlers: self.actions,
            }
        }
    }
}

/// Builder of CommitAndUndelegate Intent
pub struct CommitAndUndelegateIntentBuilder<'a, 'info> {
    accounts: &'a [AccountInfo<'info>],
    post_commit_actions: Vec<CallHandler<'info>>,
    post_undelegate_actions: Vec<CallHandler<'info>>,
}

impl<'a, 'info> CommitAndUndelegateIntentBuilder<'a, 'info> {
    pub fn new(accounts: &'a [AccountInfo<'info>]) -> Self {
        Self {
            accounts,
            post_commit_actions: vec![],
            post_undelegate_actions: vec![],
        }
    }

    pub fn add_post_commit_actions(
        mut self,
        actions: impl IntoIterator<Item = CallHandler<'info>>,
    ) -> Self {
        self.post_commit_actions.extend(actions);
        self
    }

    pub fn add_post_undelegate_actions(
        mut self,
        actions: impl IntoIterator<Item = CallHandler<'info>>,
    ) -> Self {
        self.post_undelegate_actions.extend(actions);
        self
    }

    pub fn build(self) -> CommitAndUndelegate<'info> {
        let commit_type = CommitIntentBuilder::new(self.accounts)
            .add_post_commit_actions(self.post_commit_actions)
            .build();
        let undelegate_type = if self.post_undelegate_actions.is_empty() {
            UndelegateType::Standalone
        } else {
            UndelegateType::WithHandler(self.post_undelegate_actions)
        };

        CommitAndUndelegate {
            commit_type,
            undelegate_type,
        }
    }
}

fn asd<'info>(accounts: &[AccountInfo<'info>]) {
    let ([payer, magic_context, magic_program, pda1, pda2, pda3], others) = accounts.split_at(3);
    MagicIntentBundleBuilder::new(payer.clone(), magic_context.clone(), magic_program.clone())
        .commit(&[pda1.clone()])
        .add_actions([])
        .commit_and_undelegate(&[pda3.clone()])
        .add_post_commit_actions([])
        .add_post_undelegate_actions([])
        .done()
        .build_and_invoke();

    todo!()
}

// impl<'info> MagicBaseIntent<'info> {
//     fn committed_accounts(&self) -> Option<&[AccountInfo<'info>]> {
//         if let Self::CommitAccounts(ref value) = self {
//             Some(value.accounts.as_slice())
//         } else {
//             None
//         }
//     }
//
//     fn undelegated_accounts(&self) -> Option<&[AccountInfo<'info>]> {
//         if let Self::CommitAndUndelegateAccounts(ref value) = self {
//             Some(value.accounts.as_slice())
//         } else {
//             None
//         }
//     }
//
//     fn collect_accounts(&self, container: &mut Vec<AccountInfo<'info>>) {
//         match self {
//             Self::StandaloneActions(actions) => {
//                 actions
//                     .iter()
//                     .for_each(|action| action.collect_accounts(container));
//             }
//             Self::CommitAccounts(value) => {
//                 container.extend(value.accounts.iter().cloned());
//                 value
//                     .actions
//                     .iter()
//                     .for_each(|action| action.collect_accounts(container));
//             }
//             Self::CommitAndUndelegateAccounts(value) => {
//                 container.extend(value.accounts.iter().cloned());
//                 value
//                     .post_commit_actions
//                     .iter()
//                     .for_each(|action| action.collect_accounts(container));
//                 value
//                     .post_undelegate_actions
//                     .iter()
//                     .for_each(|action| action.collect_accounts(container));
//             }
//         }
//     }
// }

fn kek<'info>(intents: Vec<MagicBaseIntent<'info>>) {
    #[derive(Default)]
    struct MergedAccounts<'info> {
        committed: Vec<AccountInfo<'info>>,
        undelegated: Vec<AccountInfo<'info>>,
    };

    let mut merged = intents
        .iter()
        .fold(MergedAccounts::default(), |mut merged, intent| {
            if let Some(accounts) = intent.committed_accounts() {
                merged.committed.extend_from_slice(accounts);
            }
            if let Some(accounts) = intent.undelegated_accounts() {
                merged.undelegated.extend_from_slice(accounts);
            }
            merged
        });

    // Sort accounts
    merged.committed.sort_by(|a, b| a.key.cmp(b.key));
    merged.undelegated.sort_by(|a, b| a.key.cmp(b.key));

    // Dedup them
    merged.committed.dedup_by(|a, b| a.key.eq(b.key));
    merged.undelegated.dedup_by(|a, b| a.key.eq(b.key));

    let (committed, undelegated) = merge(merged.committed, merged.undelegated);
}

fn merge<'info>(
    mut a: Vec<AccountInfo<'info>>,
    mut b: Vec<AccountInfo<'info>>,
) -> (Vec<AccountInfo<'info>>, Vec<AccountInfo<'info>>) {
    let a_len = a.len();
    let b_len = b.len();

    let mut i = 0;
    let mut j = 0;
    let mut repeated_keys = VecDeque::new();
    while i < a_len && j < b_len {
        match a[i].key.cmp(b[j].key) {
            Ordering::Less => i += 1,
            Ordering::Greater => j += 1,
            Ordering::Equal => {
                // We insert i since in case acc present in both
                // we undelegate it
                repeated_keys.push_back(*a[i].key);
                i += 1;
                j += 1;
            }
        }
    }

    // Remove duplicates with b
    a.retain(|acc| {
        if let Some(ind) = repeated_keys.front() {
            if ind == acc.key {
                repeated_keys.pop_front();
                false
            } else {
                true
            }
        } else {
            true
        }
    });

    (a, b)
}

/// User can issue multiple Intents:
/// 1. I want to only commit
/// 2. I want to commit & undelegate other
/// 3. I want to issue Standalone actions
/// 4. I want to CommitAndFinalize
///
/// Those are combined as IntentBundle
/// CommitAndFinalize Intent may restrict creation of other Intents
/// but generally Intent A, shouldn't constraint Intent B
///
/// ComitAndUndelegate but with actions after commit or undelegate
/// This seems to be a part of Intent as without it might not be valid
///
/// The arguments shall be a representation of the model
/// For now it seems that Multiple Commit Intents are unified in one big one
///
/// How 1 Commit and another Commit Intent is merged in case of overlaps
///
/// IDEA: Have Option<CommitIntent>, Option<CommitAndUndelegateIntent>
/// on user calling `commit(), `create` or `expand` existing one
///
///
fn jej() {}
