use crate::ephem::action_builder::ActionBuilder;
use crate::ephem::{
    ActionCallback, CallHandler, CommitAndUndelegate, CommitType, FoldableIntentBuilder,
    IntentInstructions, MagicIntent, MagicIntentBundleBuilder, UndelegateType,
};
use crate::solana_compat::solana::{AccountInfo, ProgramResult};

/// Builder of CommitAndUndelegate Intent.
///
/// Created via [`MagicIntentBundleBuilder::commit_and_undelegate()`] or
/// [`CommitIntentBuilder::commit_and_undelegate()`]. Owns the parent builder
/// and returns it (or a sibling sub-builder) on every transition/terminal call.
pub struct CommitAndUndelegateIntentBuilder<'info> {
    pub(in crate::ephem) parent: MagicIntentBundleBuilder<'info>,
    pub(in crate::ephem) accounts: Vec<AccountInfo<'info>>,
    pub(in crate::ephem) post_commit_actions: Vec<CallHandler<'info>>,
    pub(in crate::ephem) post_commit_callbacks: Vec<Option<ActionCallback>>,
    pub(in crate::ephem) post_undelegate_actions: Vec<CallHandler<'info>>,
    pub(in crate::ephem) post_undelegate_callbacks: Vec<Option<ActionCallback>>,
}

impl<'info> CommitAndUndelegateIntentBuilder<'info> {
    /// Adds post-commit actions. Chainable.
    pub fn add_post_commit_actions(
        mut self,
        actions: impl IntoIterator<Item = CallHandler<'info>>,
    ) -> Self {
        let actions: Vec<_> = actions.into_iter().collect();
        self.post_commit_callbacks
            .extend((0..actions.len()).map(|_| None));
        self.post_commit_actions.extend(actions);
        self
    }

    /// Adds a single post-commit action with a callback. Chainable.
    pub fn add_post_commit_action(
        self,
        action: CallHandler<'info>,
    ) -> ActionBuilder<
        'info,
        CommitAndUndelegateIntentBuilder<'info>,
        impl FnOnce(
            CommitAndUndelegateIntentBuilder<'info>,
            CallHandler<'info>,
            Option<ActionCallback>,
        ) -> CommitAndUndelegateIntentBuilder<'info>,
    > {
        ActionBuilder::new(
            self,
            action,
            |mut parent: CommitAndUndelegateIntentBuilder<'info>, action, callback| {
                parent.post_commit_actions.push(action);
                parent.post_commit_callbacks.push(callback);
                parent
            },
        )
    }

    /// Adds post-undelegate actions. Chainable.
    pub fn add_post_undelegate_actions(
        mut self,
        actions: impl IntoIterator<Item = CallHandler<'info>>,
    ) -> Self {
        let actions: Vec<_> = actions.into_iter().collect();
        self.post_undelegate_callbacks
            .extend((0..actions.len()).map(|_| None));
        self.post_undelegate_actions.extend(actions);
        self
    }

    /// Adds a single post-undelegate action. Returns an [`ActionBuilder`] that lets you
    /// optionally attach a callback via `.then()` before continuing the chain.
    pub fn add_post_undelegate_action(
        self,
        action: CallHandler<'info>,
    ) -> ActionBuilder<
        'info,
        CommitAndUndelegateIntentBuilder<'info>,
        impl FnOnce(
            CommitAndUndelegateIntentBuilder<'info>,
            CallHandler<'info>,
            Option<ActionCallback>,
        ) -> CommitAndUndelegateIntentBuilder<'info>,
    > {
        ActionBuilder::new(
            self,
            action,
            |mut parent: CommitAndUndelegateIntentBuilder<'info>, action, callback| {
                parent.post_undelegate_actions.push(action);
                parent.post_undelegate_callbacks.push(callback);
                parent
            },
        )
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

impl<'info> FoldableIntentBuilder<'info> for CommitAndUndelegateIntentBuilder<'info> {
    fn fold_builder(self) -> MagicIntentBundleBuilder<'info> {
        let Self {
            mut parent,
            accounts,
            post_commit_actions,
            post_commit_callbacks,
            post_undelegate_actions,
            post_undelegate_callbacks,
        } = self;
        let commit_type = if post_commit_actions.is_empty() {
            CommitType::Standalone(accounts)
        } else {
            CommitType::WithHandler {
                commited_accounts: accounts,
                call_handlers: post_commit_actions,
                callbacks: post_commit_callbacks,
            }
        };
        let undelegate_type = if post_undelegate_actions.is_empty() {
            UndelegateType::Standalone
        } else {
            UndelegateType::WithHandler {
                call_handlers: post_undelegate_actions,
                callbacks: post_undelegate_callbacks,
            }
        };
        let cau = CommitAndUndelegate {
            commit_type,
            undelegate_type,
        };
        parent
            .intent_bundle
            .add_intent(MagicIntent::CommitAndUndelegate(cau));
        parent
    }
}

pub trait FoldableCauIntentBuilder<'info>: FoldableIntentBuilder<'info> {
    fn fold_cau_builder(self) -> CommitAndUndelegateIntentBuilder<'info>;

    /// Adds post-commit actions. Chainable.
    fn add_post_commit_actions(
        self,
        actions: impl IntoIterator<Item = CallHandler<'info>>,
    ) -> CommitAndUndelegateIntentBuilder<'info> {
        self.fold_cau_builder().add_post_commit_actions(actions)
    }

    /// Adds a single post-commit action with a callback. Chainable.
    fn add_post_commit_action(
        self,
        action: CallHandler<'info>,
    ) -> ActionBuilder<
        'info,
        CommitAndUndelegateIntentBuilder<'info>,
        impl FnOnce(
            CommitAndUndelegateIntentBuilder<'info>,
            CallHandler<'info>,
            Option<ActionCallback>,
        ) -> CommitAndUndelegateIntentBuilder<'info>,
    > {
        self.fold_cau_builder().add_post_commit_action(action)
    }

    /// Adds post-undelegate actions. Chainable.
    fn add_post_undelegate_actions(
        self,
        actions: impl IntoIterator<Item = CallHandler<'info>>,
    ) -> CommitAndUndelegateIntentBuilder<'info> {
        self.fold_cau_builder().add_post_undelegate_actions(actions)
    }

    /// Adds a single post-undelegate action with a callback. Chainable.
    fn add_post_undelegate_action(
        self,
        action: CallHandler<'info>,
    ) -> ActionBuilder<
        'info,
        CommitAndUndelegateIntentBuilder<'info>,
        impl FnOnce(
            CommitAndUndelegateIntentBuilder<'info>,
            CallHandler<'info>,
            Option<ActionCallback>,
        ) -> CommitAndUndelegateIntentBuilder<'info>,
    > {
        self.fold_cau_builder().add_post_undelegate_action(action)
    }
}
