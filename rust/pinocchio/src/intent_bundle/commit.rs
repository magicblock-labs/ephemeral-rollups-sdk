use crate::intent_bundle::commit_and_undelegate::CommitAndUndelegateIntentBuilder;
use crate::intent_bundle::no_vec::NoVec;
use crate::intent_bundle::types::MagicIntentBundle;
use crate::intent_bundle::{CallHandler, CommitIntent, MagicIntent, MagicIntentBundleBuilder};
use pinocchio::cpi::MAX_STATIC_CPI_ACCOUNTS;
use pinocchio::error::ProgramError;
use pinocchio::{AccountView, ProgramResult};

/// Builder of Commit Intent.
///
/// Created via [`MagicIntentBundleBuilder::commit()`]. Owns the parent builder
/// and returns it (or a sibling sub-builder) on every transition/terminal call.
pub struct CommitIntentBuilder<'a, 'pa, 'args, T> {
    parent: MagicIntentBundleBuilder<'pa, 'args>,
    accounts: &'a [AccountView],
    actions: T,
}

impl<'a, 'pa, 'args> CommitIntentBuilder<'a, 'pa, 'args, &'static [CallHandler<'static>]> {
    pub fn new(parent: MagicIntentBundleBuilder<'pa, 'args>, accounts: &'a [AccountView]) -> Self {
        Self {
            parent,
            accounts,
            actions: &[],
        }
    }

    /// Adds post-commit actions
    pub fn add_post_commit_actions<'slice, 'new_args>(
        self,
        actions: &'slice [CallHandler<'new_args>],
    ) -> MagicIntentBundleBuilder<'slice, 'new_args>
    where
        'args: 'new_args,
        'pa: 'slice,
    {
        let MagicIntentBundle {
            standalone_actions,
            commit_intent: _,
            commit_and_undelegate_intent,
        } = self.parent.intent_bundle;

        let mut accounts = NoVec::new();
        accounts.append_slice(self.accounts);
        MagicIntentBundleBuilder {
            payer: self.parent.payer,
            magic_context: self.parent.magic_context,
            magic_program: self.parent.magic_program,
            intent_bundle: MagicIntentBundle {
                standalone_actions,
                commit_intent: Some(CommitIntent { accounts, actions }),
                commit_and_undelegate_intent,
            },
        }
    }
}

/// `a - lifetime of slice &[AccountView[
/// `pa - lifetime of slice [CallHandler] in parent builder
/// `args - lifetime of CallHandler args slice
/// `new_args - new lifetime of new CallHandler args slice
/// `slice - new lifetime of slice [CallHandler] in parent builder
impl<'a, 'pa, 'args, 'new_args, 'slice>
    CommitIntentBuilder<'a, 'pa, 'args, &'slice [CallHandler<'new_args>]>
where
    'args: 'new_args,
    'pa: 'slice,
{
    /// Finalizes this commit intent and folds it into the parent bundle.
    pub fn fold(self) -> MagicIntentBundleBuilder<'slice, 'new_args> {
        let mut accounts = NoVec::new();
        accounts.append_slice(self.accounts);

        let MagicIntentBundle {
            standalone_actions,
            commit_intent: _,
            commit_and_undelegate_intent,
        } = self.parent.intent_bundle;
        let commit_intent = Some(CommitIntent {
            accounts,
            actions: self.actions,
        });
        MagicIntentBundleBuilder {
            payer: self.parent.payer,
            magic_context: self.parent.magic_context,
            magic_program: self.parent.magic_program,
            intent_bundle: MagicIntentBundle {
                standalone_actions,
                commit_intent,
                commit_and_undelegate_intent,
            },
        }
    }

    /// Transition: finalizes this commit intent and starts a commit-and-undelegate intent.
    pub fn commit_and_undelegate<'cau>(
        self,
        accounts: &'cau [AccountView],
    ) -> CommitAndUndelegateIntentBuilder<
        'cau,
        'slice,
        'new_args,
        &'static [CallHandler<'static>],
        &'static [CallHandler<'static>],
    > {
        self.fold().commit_and_undelegate(accounts)
    }

    /// Transition: finalizes this commit intent and adds standalone base-layer actions.
    pub fn add_standalone_actions<'new_slice, 'new_args>(
        self,
        actions: &'new_slice [CallHandler<'new_args>],
    ) -> MagicIntentBundleBuilder<'new_slice, 'new_args>
    where
        'args: 'new_args,
    {
        self.fold().add_standalone_actions(actions)
    }

    /// Terminal: finalizes this commit intent, builds the instruction and invokes it.
    pub fn build_and_invoke(self, data_buf: &mut [u8]) -> ProgramResult {
        self.done()?.build_and_invoke(data_buf)
    }
}
