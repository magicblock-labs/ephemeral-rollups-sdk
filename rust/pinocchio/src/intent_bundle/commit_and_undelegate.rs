use pinocchio::{AccountView, ProgramResult};

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
/// - `'acc`  – lifetime of the `&[AccountView]` slice passed to `.commit_and_undelegate()`
/// - `'args` – lifetime of `&[CallHandler]` action slices and their payload data
/// - `S1`    – typestate: tracks whether post-commit actions have been set
/// - `S2`    – typestate: tracks whether post-undelegate actions have been set
pub struct CommitAndUndelegateIntentBuilder<'acc, 'args, S1, S2> {
    parent: MagicIntentBundleBuilder<'acc, 'args>,
    accounts: &'acc [AccountView],
    post_commit_actions: S1,
    post_undelegate_actions: S2,
}

/// Builder without actions
impl<'acc, 'args>
    CommitAndUndelegateIntentBuilder<
        'acc,
        'args,
        &'static [CallHandler<'static>],
        &'static [CallHandler<'static>],
    >
{
    pub fn new<'new_acc>(
        parent: MagicIntentBundleBuilder<'acc, 'args>,
        accounts: &'new_acc [AccountView],
    ) -> CommitAndUndelegateIntentBuilder<
        'new_acc,
        'args,
        &'static [CallHandler<'static>],
        &'static [CallHandler<'static>],
    >
    where
        'acc: 'new_acc,
    {
        CommitAndUndelegateIntentBuilder {
            parent,
            accounts,
            post_commit_actions: &[],
            post_undelegate_actions: &[],
        }
    }
}

/// Builder with post_commit_actions not defined yet
impl<'acc, 'args, S2>
    CommitAndUndelegateIntentBuilder<'acc, 'args, &'static [CallHandler<'static>], S2>
{
    /// Adds post-commit actions. Chainable.
    pub fn add_post_commit_actions<'new_args>(
        self,
        actions: &'new_args [CallHandler<'new_args>],
    ) -> CommitAndUndelegateIntentBuilder<'acc, 'new_args, &'new_args [CallHandler<'new_args>], S2>
    where
        'args: 'new_args,
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
impl<'acc, 'args, T1>
    CommitAndUndelegateIntentBuilder<'acc, 'args, T1, &'static [CallHandler<'static>]>
{
    /// Adds post-undelegate actions. Chainable.
    pub fn add_post_undelegate_actions<'new_args>(
        self,
        actions: &'new_args [CallHandler<'new_args>],
    ) -> CommitAndUndelegateIntentBuilder<'acc, 'new_args, T1, &'new_args [CallHandler<'new_args>]>
    where
        'args: 'new_args,
    {
        CommitAndUndelegateIntentBuilder {
            parent: self.parent,
            accounts: self.accounts,
            post_commit_actions: self.post_commit_actions,
            post_undelegate_actions: actions,
        }
    }
}

impl<'acc, 'args>
    CommitAndUndelegateIntentBuilder<
        'acc,
        'args,
        &'args [CallHandler<'args>],
        &'args [CallHandler<'args>],
    >
{
    /// Transition: finalizes this commit-and-undelegate intent and starts a new commit intent.
    pub fn commit<'new_acc>(
        self,
        accounts: &'new_acc [AccountView],
    ) -> CommitIntentBuilder<'new_acc, 'args, &'static [CallHandler<'static>]>
    where
        'acc: 'new_acc,
    {
        self.fold().commit(accounts)
    }

    /// Transition: finalizes this commit-and-undelegate intent and adds standalone base-layer actions.
    pub fn set_standalone_actions<'new_args>(
        self,
        actions: &'new_args [CallHandler<'new_args>],
    ) -> MagicIntentBundleBuilder<'acc, 'new_args>
    where
        'args: 'new_args,
    {
        self.fold().set_standalone_actions(actions)
    }

    /// Terminal: finalizes this intent, builds the instruction and invokes it.
    pub fn build_and_invoke(self, data_buf: &mut [u8]) -> ProgramResult {
        self.fold().build_and_invoke(data_buf)
    }

    /// Finalizes this commit-and-undelegate intent and folds it into the parent bundle.
    pub fn fold(self) -> MagicIntentBundleBuilder<'acc, 'args> {
        let Self {
            accounts,
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
