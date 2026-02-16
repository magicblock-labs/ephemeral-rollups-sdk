use crate::intent_bundle::commit_and_undelegate::CommitAndUndelegateIntentBuilder;
use crate::intent_bundle::no_vec::NoVec;
use crate::intent_bundle::types::MagicIntentBundle;
use crate::intent_bundle::{CallHandler, CommitIntent, MagicIntentBundleBuilder};
use pinocchio::{AccountView, ProgramResult};

/// Builder of Commit Intent.
///
/// Created via [`MagicIntentBundleBuilder::commit()`]. Owns the parent builder
/// and returns it (or a sibling sub-builder) on every transition/terminal call.
///
/// - `'act`  – lifetime of `&[CallHandler]` action slices stored in the parent bundle
/// - `'args` – lifetime of the data inside `CallHandler` (i.e. `ActionArgs` payload)
/// - `'acc`  – lifetime of the `&[AccountView]` slice passed to `.commit()`
/// - `T`     – typestate: tracks whether post-commit actions have been set
pub struct CommitIntentBuilder<'act, 'args, 'acc, T> {
    parent: MagicIntentBundleBuilder<'act, 'args>,
    accounts: &'acc [AccountView],
    actions: T,
}

impl<'act, 'args, 'acc> CommitIntentBuilder<'act, 'args, 'acc, &'static [CallHandler<'static>]> {
    pub fn new(
        parent: MagicIntentBundleBuilder<'act, 'args>,
        accounts: &'acc [AccountView],
    ) -> Self {
        Self {
            parent,
            accounts,
            actions: &[],
        }
    }

    /// Adds post-commit actions
    pub fn add_post_commit_actions<'new_act, 'new_args>(
        self,
        actions: &'new_act [CallHandler<'new_args>],
    ) -> MagicIntentBundleBuilder<'new_act, 'new_args>
    where
        'args: 'new_args,
        'act: 'new_act,
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

/// - `'act`      – lifetime of `&[CallHandler]` action slices in the parent bundle
/// - `'args`     – lifetime of the `CallHandler` args payload
/// - `'acc`      – lifetime of the `&[AccountView]` accounts slice
/// - `'new_act`  – lifetime of a new `&[CallHandler]` slice being added
/// - `'new_args` – lifetime of the new `CallHandler` args payload
impl<'act, 'args> CommitIntentBuilder<'act, 'args, '_, &'act [CallHandler<'args>]> {
    /// Finalizes this commit intent and folds it into the parent bundle.
    pub fn fold(self) -> MagicIntentBundleBuilder<'act, 'args> {
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
    pub fn commit_and_undelegate<'cau_acc>(
        self,
        accounts: &'cau_acc [AccountView],
    ) -> CommitAndUndelegateIntentBuilder<
        'act,
        'args,
        'cau_acc,
        &'static [CallHandler<'static>],
        &'static [CallHandler<'static>],
    > {
        self.fold().commit_and_undelegate(accounts)
    }

    /// Transition: finalizes this commit intent and adds standalone base-layer actions.
    pub fn set_standalone_actions<'new_act, 'new_args>(
        self,
        actions: &'new_act [CallHandler<'new_args>],
    ) -> MagicIntentBundleBuilder<'new_act, 'new_args>
    where
        'args: 'new_args,
        'act: 'new_act,
    {
        self.fold().set_standalone_actions(actions)
    }

    /// Terminal: finalizes this commit intent, builds the instruction and invokes it.
    pub fn build_and_invoke(self, data_buf: &mut [u8]) -> ProgramResult {
        self.fold().build_and_invoke(data_buf)
    }
}
