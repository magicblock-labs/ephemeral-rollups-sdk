use crate::intent_bundle::commit_and_undelegate::CommitAndUndelegateIntentBuilder;
use crate::intent_bundle::types::MagicIntentBundle;
use crate::intent_bundle::{CallHandler, CommitIntent, MagicIntentBundleBuilder};
use pinocchio::{AccountView, ProgramResult};

/// Builder of Commit Intent.
///
/// Created via [`MagicIntentBundleBuilder::commit()`]. Owns the parent builder
/// and returns it (or a sibling sub-builder) on every transition/terminal call.
///
/// - `'acc`  – lifetime of the `&[AccountView]` slice passed to `.commit()`
/// - `'args` – lifetime of `&[CallHandler]` action slices and their payload data
/// - `T`     – typestate: tracks whether post-commit actions have been set
pub struct CommitIntentBuilder<'acc, 'args, T> {
    parent: MagicIntentBundleBuilder<'acc, 'args>,
    accounts: &'acc [AccountView],
    actions: T,
}

impl<'acc, 'args> CommitIntentBuilder<'acc, 'args, &'static [CallHandler<'static>]> {
    pub fn new<'new_acc>(
        parent: MagicIntentBundleBuilder<'acc, 'args>,
        accounts: &'new_acc [AccountView],
    ) -> CommitIntentBuilder<'new_acc, 'args, &'static [CallHandler<'static>]>
    where
        'acc: 'new_acc,
    {
        CommitIntentBuilder {
            parent,
            accounts,
            actions: &[],
        }
    }

    /// Adds post-commit actions and folds this intent into the parent bundle.
    pub fn add_post_commit_actions<'new_args>(
        self,
        actions: &'new_args [CallHandler<'new_args>],
    ) -> MagicIntentBundleBuilder<'acc, 'new_args>
    where
        'args: 'new_args,
    {
        let MagicIntentBundle {
            standalone_actions,
            commit_intent: _,
            commit_and_undelegate_intent,
        } = self.parent.intent_bundle;

        MagicIntentBundleBuilder {
            payer: self.parent.payer,
            magic_context: self.parent.magic_context,
            magic_program: self.parent.magic_program,
            magic_fee_vault: self.parent.magic_fee_vault,
            intent_bundle: MagicIntentBundle {
                standalone_actions,
                commit_intent: Some(CommitIntent {
                    accounts: self.accounts,
                    actions,
                }),
                commit_and_undelegate_intent,
            },
        }
    }
}

impl<'acc, 'args> CommitIntentBuilder<'acc, 'args, &'args [CallHandler<'args>]> {
    /// Finalizes this commit intent and folds it into the parent bundle.
    pub fn fold(self) -> MagicIntentBundleBuilder<'acc, 'args> {
        let MagicIntentBundle {
            standalone_actions,
            commit_intent: _,
            commit_and_undelegate_intent,
        } = self.parent.intent_bundle;
        let commit_intent = Some(CommitIntent {
            accounts: self.accounts,
            actions: self.actions,
        });
        MagicIntentBundleBuilder {
            payer: self.parent.payer,
            magic_context: self.parent.magic_context,
            magic_program: self.parent.magic_program,
            magic_fee_vault: self.parent.magic_fee_vault,
            intent_bundle: MagicIntentBundle {
                standalone_actions,
                commit_intent,
                commit_and_undelegate_intent,
            },
        }
    }

    /// Transition: finalizes this commit intent and starts a commit-and-undelegate intent.
    pub fn commit_and_undelegate<'new_acc>(
        self,
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
        self.fold().commit_and_undelegate(accounts)
    }

    /// Transition: finalizes this commit intent and adds standalone base-layer actions.
    pub fn set_standalone_actions<'new_args>(
        self,
        actions: &'new_args [CallHandler<'new_args>],
    ) -> MagicIntentBundleBuilder<'acc, 'new_args>
    where
        'args: 'new_args,
    {
        self.fold().set_standalone_actions(actions)
    }

    /// Terminal: finalizes this commit intent, builds the instruction and invokes it.
    pub fn build_and_invoke(self, data_buf: &mut [u8]) -> ProgramResult {
        self.fold().build_and_invoke(data_buf)
    }
}
