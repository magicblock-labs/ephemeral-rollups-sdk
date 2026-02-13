use pinocchio::cpi::MAX_STATIC_CPI_ACCOUNTS;
use pinocchio::error::ProgramError;
use pinocchio::{AccountView, ProgramResult};

use crate::intent_bundle::no_vec::NoVec;
use crate::intent_bundle::types::MagicIntentBundle;
use crate::intent_bundle::{
    CallHandler, CommitAndUndelegateIntent, CommitIntentBuilder, MagicIntent,
    MagicIntentBundleBuilder,
};

/// Builder of CommitAndUndelegate Intent.
///
/// Created via [`MagicIntentBundleBuilder::commit_and_undelegate()`] or
/// [`CommitIntentBuilder::commit_and_undelegate()`]. Owns the parent builder
/// and returns it (or a sibling sub-builder) on every transition/terminal call.
pub struct CommitAndUndelegateIntentBuilder<'a, 'pa, 'args, S1, S2> {
    parent: MagicIntentBundleBuilder<'pa, 'args>,
    accounts: &'a [AccountView],
    post_commit_actions: S1,
    post_undelegate_actions: S2,
}

/// Builder without actions
impl<'a, 'pa, 'args>
    CommitAndUndelegateIntentBuilder<
        'a,
        'pa,
        'args,
        &'static [CallHandler<'static>],
        &'static [CallHandler<'static>],
    >
{
    pub fn new(parent: MagicIntentBundleBuilder<'pa, 'args>, accounts: &'a [AccountView]) -> Self {
        Self {
            parent,
            accounts,
            post_commit_actions: &[],
            post_undelegate_actions: &[],
        }
    }
}

/// Builder with post_commit_actions not defined yet
impl<'a, 'pa, 'args, S2>
    CommitAndUndelegateIntentBuilder<'a, 'pa, 'args, &'static [CallHandler<'static>], S2>
{
    /// Adds post-undelegate actions. Chainable.
    pub fn add_post_commit_actions<'new_args, 'slice>(
        self,
        actions: &'slice [CallHandler<'new_args>],
    ) -> CommitAndUndelegateIntentBuilder<'a, 'slice, 'new_args, &'slice [CallHandler<'new_args>], S2>
    where
        'args: 'new_args,
        'pa: 'slice,
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
impl<'a, 'pa, 'args, T1>
    CommitAndUndelegateIntentBuilder<'a, 'pa, 'args, T1, &'static [CallHandler<'static>]>
{
    /// Adds post-undelegate actions. Chainable.
    pub fn add_post_undelegate_actions<'new_args, 'other_slice>(
        self,
        actions: &'other_slice [CallHandler<'new_args>],
    ) -> CommitAndUndelegateIntentBuilder<
        'a,
        'other_slice,
        'new_args,
        T1,
        &'other_slice [CallHandler<'new_args>],
    >
    where
        'args: 'new_args,
        'pa: 'other_slice,
    {
        CommitAndUndelegateIntentBuilder {
            parent: self.parent,
            accounts: self.accounts,
            post_commit_actions: self.post_commit_actions,
            post_undelegate_actions: actions,
        }
    }
}

impl<'a, 'pa, 'args>
    CommitAndUndelegateIntentBuilder<
        'a,
        'pa,
        'args,
        &'pa [CallHandler<'args>],
        &'pa [CallHandler<'args>],
    >
{
    /// Transition: finalizes this commit-and-undelegate intent and starts a new commit intent.
    pub fn commit<'b>(
        self,
        accounts: &'b [AccountView],
    ) -> CommitIntentBuilder<'b, 'pa, 'args, &'static [CallHandler<'static>]> {
        self.fold().commit(accounts)
    }

    /// Transition: finalizes this commit-and-undelegate intent and adds standalone base-layer actions.
    pub fn add_standalone_actions<'new_a, 'newargs>(
        self,
        actions: &'new_a [CallHandler<'newargs>],
    ) -> MagicIntentBundleBuilder<'new_a, 'newargs>
    where
        'args: 'newargs,
        'pa: 'new_a,
    {
        self.fold().add_standalone_actions(actions)
    }

    /// Terminal: finalizes this intent, builds the instruction and invokes it.
    pub fn build_and_invoke(self, data_buf: &mut [u8]) -> ProgramResult {
        self.fold().build_and_invoke(data_buf)
    }

    /// Finalizes this commit-and-undelegate intent and folds it into the parent bundle.
    pub fn fold(self) -> MagicIntentBundleBuilder<'pa, 'args> {
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
