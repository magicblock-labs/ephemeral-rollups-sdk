use crate::ephem::action_builder::ActionBuilder;
use crate::ephem::{
    ActionCallback, CallHandler, CommitType, FoldableIntentBuilder, IntentInstructions,
    MagicIntent, MagicIntentBundleBuilder,
};
use crate::solana_compat::solana::{AccountInfo, ProgramResult};

/// Builder of Commit Intent.
///
/// Created via [`MagicIntentBundleBuilder::commit()`]. Owns the parent builder
/// and returns it (or a sibling sub-builder) on every transition/terminal call.
pub struct CommitIntentBuilder<'info> {
    pub(in crate::ephem) parent: MagicIntentBundleBuilder<'info>,
    pub(in crate::ephem) accounts: Vec<AccountInfo<'info>>,
    pub(in crate::ephem) actions: Vec<CallHandler<'info>>,
    pub(in crate::ephem) callbacks: Vec<Option<ActionCallback>>,
}

impl<'info> CommitIntentBuilder<'info> {
    /// Adds post-commit actions. Chainable.
    pub fn add_post_commit_actions(
        mut self,
        actions: impl IntoIterator<Item = CallHandler<'info>>,
    ) -> Self {
        let actions: Vec<_> = actions.into_iter().collect();
        self.callbacks.extend((0..actions.len()).map(|_| None));
        self.actions.extend(actions);
        self
    }

    /// Adds a single post-commit action. Returns an [`ActionBuilder`] that lets you
    /// optionally attach a callback via `.then()` before continuing the chain.
    pub fn add_post_commit_action(
        self,
        action: CallHandler<'info>,
    ) -> ActionBuilder<
        'info,
        CommitIntentBuilder<'info>,
        impl FnOnce(
            CommitIntentBuilder<'info>,
            CallHandler<'info>,
            Option<ActionCallback>,
        ) -> CommitIntentBuilder<'info>,
    > {
        ActionBuilder::new(self, action, |mut parent, action, callback| {
            parent.actions.push(action);
            parent.callbacks.push(callback);
            parent
        })
    }

    /// Builds [`IntentInstructions`] without invoking them.
    ///
    /// Inherent forwarder to [`FoldableIntentBuilder::build`] so callers
    /// don't need to import the trait.
    pub fn build(self) -> IntentInstructions<'info> {
        <Self as FoldableIntentBuilder>::build(self)
    }

    /// Builds and invokes the intent bundle via CPI.
    ///
    /// Inherent forwarder to [`FoldableIntentBuilder::build_and_invoke`] so callers
    /// don't need to import the trait.
    pub fn build_and_invoke(self) -> ProgramResult {
        <Self as FoldableIntentBuilder>::build_and_invoke(self)
    }

    /// Builds and invokes the intent bundle via signed CPI using the provided PDA seeds.
    ///
    /// Inherent forwarder to [`FoldableIntentBuilder::build_and_invoke_signed`] so callers
    /// don't need to import the trait.
    pub fn build_and_invoke_signed(self, signers_seeds: &[&[&[u8]]]) -> ProgramResult {
        <Self as FoldableIntentBuilder>::build_and_invoke_signed(self, signers_seeds)
    }
}

impl<'info> FoldableIntentBuilder<'info> for CommitIntentBuilder<'info> {
    fn fold_builder(self) -> MagicIntentBundleBuilder<'info> {
        let Self {
            mut parent,
            accounts,
            actions,
            callbacks,
        } = self;
        let commit = if actions.is_empty() {
            CommitType::Standalone(accounts)
        } else {
            CommitType::WithHandler {
                commited_accounts: accounts,
                call_handlers: actions,
                callbacks,
            }
        };
        parent.intent_bundle.add_intent(MagicIntent::Commit(commit));
        parent
    }
}

/// Shared transition methods for builders that wrap a [`CommitIntentBuilder`].
///
/// Implemented by [`ActionBuilder`] so that callers can chain further commit actions
/// or transition to other intents without manually calling `fold_commit_builder()`.
pub trait FoldableCommitIntentBuilder<'info>: FoldableIntentBuilder<'info> {
    fn fold_commit_builder(self) -> CommitIntentBuilder<'info>;

    /// Adds post-commit actions. Chainable.
    fn add_post_commit_actions(
        self,
        actions: impl IntoIterator<Item = CallHandler<'info>>,
    ) -> CommitIntentBuilder<'info> {
        self.fold_commit_builder().add_post_commit_actions(actions)
    }

    /// Adds a single post-commit action. Returns an [`ActionBuilder`] for optional callback attachment.
    fn add_post_commit_action(
        self,
        action: CallHandler<'info>,
    ) -> ActionBuilder<
        'info,
        CommitIntentBuilder<'info>,
        impl FnOnce(
            CommitIntentBuilder<'info>,
            CallHandler<'info>,
            Option<ActionCallback>,
        ) -> CommitIntentBuilder<'info>,
    > {
        self.fold_commit_builder().add_post_commit_action(action)
    }
}
