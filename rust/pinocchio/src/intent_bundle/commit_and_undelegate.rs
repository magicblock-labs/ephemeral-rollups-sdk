use pinocchio::cpi::MAX_STATIC_CPI_ACCOUNTS;
use pinocchio::{AccountView, ProgramResult};

use crate::intent_bundle::no_vec::NoVec;
use crate::intent_bundle::types::MagicIntentBundle;
use crate::intent_bundle::{
    CallHandler, CommitAndUndelegateIntent, CommitIntentBuilder, MagicIntentBundleBuilder,
};

/// Builder of CommitAndUndelegate Intent.
///
/// Created via [`MagicIntentBundleBuilder::commit_and_undelegate()`] or
/// [`CommitIntentBuilder::commit_and_undelegate()`]. Owns the parent builder
/// and returns it (or a sibling sub-builder) on every transition/terminal call.
///
/// - `'act`  – lifetime of `&[CallHandler]` action slices stored in the parent bundle
/// - `'args` – lifetime of the data inside `CallHandler` (i.e. `ActionArgs` payload)
/// - `'acc`  – lifetime of the `&[AccountView]` slice passed to `.commit_and_undelegate()`
/// - `S1`    – typestate: tracks whether post-commit actions have been set
/// - `S2`    – typestate: tracks whether post-undelegate actions have been set
pub struct CommitAndUndelegateIntentBuilder<'act, 'args, 'acc, S1, S2> {
    parent: MagicIntentBundleBuilder<'act, 'args>,
    accounts: &'acc [AccountView],
    post_commit_actions: S1,
    post_undelegate_actions: S2,
}

/// Builder without actions
impl<'act, 'args, 'acc>
    CommitAndUndelegateIntentBuilder<
        'act,
        'args,
        'acc,
        &'static [CallHandler<'static>],
        &'static [CallHandler<'static>],
    >
{
    pub fn new(
        parent: MagicIntentBundleBuilder<'act, 'args>,
        accounts: &'acc [AccountView],
    ) -> Self {
        Self {
            parent,
            accounts,
            post_commit_actions: &[],
            post_undelegate_actions: &[],
        }
    }
}

/// Builder with post_commit_actions not defined yet
impl<'act, 'args, 'acc, S2>
    CommitAndUndelegateIntentBuilder<'act, 'args, 'acc, &'static [CallHandler<'static>], S2>
{
    /// Adds post-commit actions. Chainable.
    pub fn add_post_commit_actions<'new_args, 'new_act>(
        self,
        actions: &'new_act [CallHandler<'new_args>],
    ) -> CommitAndUndelegateIntentBuilder<
        'new_act,
        'new_args,
        'acc,
        &'new_act [CallHandler<'new_args>],
        S2,
    >
    where
        'args: 'new_args,
        'act: 'new_act,
    {
        CommitAndUndelegateIntentBuilder {
            parent: self.parent,
            accounts: self.accounts,
            post_commit_actions: actions,
            post_undelegate_actions: self.post_undelegate_actions,
        }
    }
}

/// Builder with post_undelegate_actions not defined yet
impl<'act, 'args, 'acc, T1>
    CommitAndUndelegateIntentBuilder<'act, 'args, 'acc, T1, &'static [CallHandler<'static>]>
{
    /// Adds post-undelegate actions. Chainable.
    pub fn add_post_undelegate_actions<'new_args, 'new_act>(
        self,
        actions: &'new_act [CallHandler<'new_args>],
    ) -> CommitAndUndelegateIntentBuilder<
        'new_act,
        'new_args,
        'acc,
        T1,
        &'new_act [CallHandler<'new_args>],
    >
    where
        'args: 'new_args,
        'act: 'new_act,
    {
        CommitAndUndelegateIntentBuilder {
            parent: self.parent,
            accounts: self.accounts,
            post_commit_actions: self.post_commit_actions,
            post_undelegate_actions: actions,
        }
    }
}

impl<'act, 'args>
    CommitAndUndelegateIntentBuilder<
        'act,
        'args,
        '_,
        &'act [CallHandler<'args>],
        &'act [CallHandler<'args>],
    >
{
    /// Transition: finalizes this commit-and-undelegate intent and starts a new commit intent.
    pub fn commit<'commit_acc>(
        self,
        accounts: &'commit_acc [AccountView],
    ) -> CommitIntentBuilder<'act, 'args, 'commit_acc, &'static [CallHandler<'static>]> {
        self.fold().commit(accounts)
    }

    /// Transition: finalizes this commit-and-undelegate intent and adds standalone base-layer actions.
    pub fn add_standalone_actions<'new_act, 'new_args>(
        self,
        actions: &'new_act [CallHandler<'new_args>],
    ) -> MagicIntentBundleBuilder<'new_act, 'new_args>
    where
        'args: 'new_args,
        'act: 'new_act,
    {
        self.fold().add_standalone_actions(actions)
    }

    /// Terminal: finalizes this intent, builds the instruction and invokes it.
    pub fn build_and_invoke(self, data_buf: &mut [u8]) -> ProgramResult {
        self.fold().build_and_invoke(data_buf)
    }

    /// Finalizes this commit-and-undelegate intent and folds it into the parent bundle.
    pub fn fold(self) -> MagicIntentBundleBuilder<'act, 'args> {
        let Self {
            accounts: committed_accounts,
            post_commit_actions,
            post_undelegate_actions,
            parent,
        } = self;
        let MagicIntentBundleBuilder {
            payer,
            magic_context,
            magic_program,
            intent_bundle,
        } = parent;
        let MagicIntentBundle {
            standalone_actions,
            commit_intent,
            commit_and_undelegate_intent: _,
        } = intent_bundle;

        let mut accounts = NoVec::<_, MAX_STATIC_CPI_ACCOUNTS>::new();
        accounts.append_slice(committed_accounts);
        let commit_and_undelegate_intent = Some(CommitAndUndelegateIntent {
            accounts,
            post_commit_actions,
            post_undelegate_actions,
        });
        MagicIntentBundleBuilder {
            payer,
            magic_context,
            magic_program,
            intent_bundle: MagicIntentBundle {
                standalone_actions,
                commit_intent,
                commit_and_undelegate_intent,
            },
        }
    }
}
