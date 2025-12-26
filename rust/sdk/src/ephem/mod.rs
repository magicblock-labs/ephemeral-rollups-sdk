use crate::ephem::deprecated::v1::{CallHandler, MagicAction};
use crate::solana_compat::solana::{AccountInfo, ProgramResult};

pub use crate::ephem::deprecated::v0::{
    commit_accounts, commit_and_undelegate_accounts, create_schedule_commit_ix,
};

pub mod deprecated;

/// The user can really do:
/// 1. Commit account
/// 2. Commit & undelegate account
/// 3. Run action
/// There's no such thing as just Undelegation
/// There's also no separate instruction on finalize
/// 1. Should we allow separate Finalize ix?
///     No, since this is a validators business. User's can't finalize an account
/// 2. Should we allow separate undelegate ix?
///     That could resolve a case of unsuccessful undelegation
///     BUT, its not clear if account failed or not, unless this info is available
///     We can't guarantee Intent execution
///     is_delegated = false, is_awaiting = true
///
///     The issue with above is that we can't tell if it failed or was just scheduled
///     and we didn't have enough time to undelegate. RACE-CONDITION
///
///     So while would be cool - `No`, unless new flag - failed_to_undelegate :-)

/// Describes types of Base Intents
pub enum BaseIntent<'info> {
    CommitAccounts(CommitIntent<'info>),
    CommitAndUndelegateAccounts(CommitAndUndelegateIntent<'info>),
    StandaloneActions(Vec<CallHandler<'info>>),
}

struct CommitIntent<'info> {
    accounts: Vec<AccountInfo<'info>>,
    actions: Vec<CallHandler<'info>>,
}

struct CommitAndUndelegateIntent<'info> {
    accounts: Vec<AccountInfo<'info>>,
    post_commit_actions: Vec<CallHandler<'info>>,
    post_undelegate_actions: Vec<CallHandler<'info>>,
}

pub struct MagicIntentsBuilder<'info> {
    payer: AccountInfo<'info>,
    magic_context: AccountInfo<'info>,
    magic_program: AccountInfo<'info>,
    intents: Vec<BaseIntent<'info>>,
}

impl<'info> MagicIntentsBuilder<'info> {
    pub fn new(
        payer: AccountInfo<'info>,
        magic_context: AccountInfo<'info>,
        magic_program: AccountInfo<'info>,
    ) -> Self {
        Self {
            payer,
            magic_context,
            magic_program,
            intents: vec![],
        }
    }

    pub fn commit<'a>(self, accounts: &'a [AccountInfo<'info>]) -> CommitIntentBuilder<'a, 'info> {
        CommitIntentBuilder::new(self, accounts)
    }

    pub fn commit_and_undelegate<'a>(
        self,
        accounts: &'a [AccountInfo<'info>],
    ) -> CommitAndUndelegateAccountsBuilder<'a, 'info> {
        CommitAndUndelegateAccountsBuilder::new(self, accounts)
    }

    pub fn build_and_invoke(self) -> ProgramResult {
        todo!()
    }
}

pub struct CommitIntentBuilder<'a, 'info> {
    builder: MagicIntentsBuilder<'info>,
    accounts: &'a [AccountInfo<'info>],
    actions: Vec<CallHandler<'info>>,
}

impl<'a, 'info> CommitIntentBuilder<'a, 'info> {
    pub fn new(builder: MagicIntentsBuilder<'info>, accounts: &'a [AccountInfo<'info>]) -> Self {
        Self {
            builder,
            accounts,
            actions: vec![],
        }
    }

    pub fn add_actions(
        mut self,
        actions: impl IntoIterator<Item = CallHandler<'info>>,
    ) -> MagicIntentsBuilder<'info> {
        self.actions.extend(actions);
        self.done()
    }

    /// Consume current builder
    /// Build Commit Intent Type and add it to `MagicIntentsBuilder`
    pub fn done(mut self) -> MagicIntentsBuilder<'info> {
        let intent = CommitIntent {
            accounts: self.accounts.to_vec(),
            actions: self.actions,
        };
        self.builder
            .intents
            .push(BaseIntent::CommitAccounts(intent));
        self.builder
    }
}

pub struct CommitAndUndelegateAccountsBuilder<'a, 'info> {
    builder: MagicIntentsBuilder<'info>,
    accounts: &'a [AccountInfo<'info>],
    post_commit_actions: Vec<CallHandler<'info>>,
    post_undelegate_actions: Vec<CallHandler<'info>>,
}

impl<'a, 'info> CommitAndUndelegateAccountsBuilder<'a, 'info> {
    pub fn new(builder: MagicIntentsBuilder<'info>, accounts: &'a [AccountInfo<'info>]) -> Self {
        Self {
            builder,
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

    pub fn done(mut self) -> MagicIntentsBuilder<'info> {
        let intent = CommitAndUndelegateIntent {
            accounts: self.accounts.to_vec(),
            post_commit_actions: self.post_commit_actions,
            post_undelegate_actions: self.post_undelegate_actions,
        };
        self.builder
            .intents
            .push(BaseIntent::CommitAndUndelegateAccounts(intent));

        self.builder
    }
}

fn asd<'info>(accounts: &[AccountInfo<'info>]) {
    let ([payer, magic_context, magic_program, pda1, pda2, pda3], others) = accounts.split_at(3);
    MagicIntentsBuilder::new(payer.clone(), magic_context.clone(), magic_program.clone())
        .commit(&[pda1.clone()])
        .add_actions([])
        .commit_and_undelegate(&[pda3.clone()])
        .add_post_commit_actions([])
        .add_post_undelegate_actions([])
        .done()
        .build_and_invoke();

    todo!()
}
