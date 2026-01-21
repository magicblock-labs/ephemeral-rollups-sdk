pub use crate::ephem::deprecated::v0::{
    commit_accounts, commit_and_undelegate_accounts, create_schedule_commit_ix,
};
use crate::ephem::deprecated::v1::{
    utils, CallHandler, CommitAndUndelegate, CommitType, MagicAction, UndelegateType,
};
use crate::solana_compat::solana::{
    invoke, AccountInfo, AccountMeta, Instruction, ProgramResult, Pubkey,
};
use std::collections::HashMap;

pub mod deprecated;

/// Describes types of Base Intents
pub type MagicBaseIntent<'info> = MagicAction<'info>;

/// Builds a single `MagicBlockInstruction::ScheduleIntentBundle` instruction by aggregating
/// multiple independent intents (base actions, commits, commit+undelegate), normalizing them,
/// and producing a deduplicated account list plus the corresponding CPI `Instruction`.
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

    /// Adds a base intent to the bundle.
    ///
    /// If an intent of the same category already exists in the bundle:
    /// - base actions are appended
    /// - commit intents are merged (accounts/actions appended; variant upgraded to handler if needed)
    /// - commit+undelegate intents are merged (accounts/actions appended)
    ///
    /// See `MagicIntentBundle::add_intent` for merge semantics.
    pub fn add_intent(mut self, intent: MagicBaseIntent<'info>) -> Self {
        self.intent_bundle.add_intent(intent);
        self
    }

    /// Adds (or merges) a `Commit` intent into the bundle.
    pub fn add_commit_intent(mut self, commit: CommitType<'info>) -> Self {
        self.intent_bundle.add_intent(MagicAction::Commit(commit));
        self
    }

    /// Adds (or merges) a `CommitAndUndelegate` intent into the bundle.
    pub fn add_commit_and_undelegate_intent(mut self, value: CommitAndUndelegate<'info>) -> Self {
        self.intent_bundle
            .add_intent(MagicAction::CommitAndUndelegate(value));
        self
    }

    /// Adds standalone base-layer actions to be executed without any commit/undelegate semantics.
    pub fn add_base_actions_intent(
        mut self,
        actions: impl IntoIterator<Item = CallHandler<'info>>,
    ) -> Self {
        self.intent_bundle
            .add_intent(MagicAction::BaseActions(actions.into_iter().collect()));
        self
    }

    /// Builds the deduplicated account list and the CPI `Instruction` that schedules this bundle.
    ///
    /// # Returns
    /// - `Vec<AccountInfo>`: the full, deduplicated account list to pass to CPI (payer/context first).
    /// - `Instruction`: the instruction to invoke against the magic program.
    pub fn build(mut self) -> (Vec<AccountInfo<'info>>, Instruction) {
        // Dedup Intent Bundle
        self.intent_bundle.normalize();

        // Coll
        let mut all_accounts = vec![self.payer, self.magic_context];
        self.intent_bundle.collect_accounts(&mut all_accounts);
        let indices_map = utils::filter_duplicates_with_map(&mut all_accounts);

        // Create data for instruction
        let args = self.intent_bundle.into_args(&indices_map);
        let metas = all_accounts
            .iter()
            .map(|ai| AccountMeta {
                pubkey: *ai.key,
                is_signer: ai.is_signer,
                is_writable: ai.is_writable,
            })
            .collect();
        let ix = Instruction::new_with_bincode(
            *self.magic_program.key,
            &MagicBlockInstruction::ScheduleIntentBundle(args),
            metas,
        );

        (all_accounts, ix)
    }

    /// Convenience wrapper that builds the instruction and immediately invokes it.
    ///
    /// Equivalent to:
    /// ```ignore
    /// let (accounts, ix) = builder.build();
    /// invoke(&ix, &accounts)
    /// ```
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
    /// Inserts an intent into the bundle, merging with any existing intent of the same category.
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

    /// Consumes the bundle and encodes it into `MagicIntentBundleArgs` using a `Pubkey -> u8` indices map.
    fn into_args(self, indices_map: &HashMap<Pubkey, u8>) -> MagicIntentBundleArgs {
        let commit = self.commit_intent.map(|c| c.into_args(&indices_map));
        let commit_and_undelegate = self
            .commit_and_undelegate_intent
            .map(|c| c.into_args(&indices_map));
        let standalone_actions = std::mem::take(&mut self.standalone_actions)
            .into_iter()
            .map(|ch| ch.into_args(&indices_map))
            .collect::<Vec<_>>();

        MagicIntentBundleArgs {
            commit,
            commit_and_undelegate,
            standalone_actions,
        }
    }

    /// Collects all accounts referenced by intents in this bundle.
    fn collect_accounts(&self, all_accounts: &mut Vec<AccountInfo<'info>>) {
        for el in &self.standalone_actions {
            el.collect_accounts(all_accounts);
        }
        if let Some(commit) = &self.commit_intent {
            commit.collect_accounts(all_accounts);
        }
        if let Some(cau) = &self.commit_and_undelegate_intent {
            cau.collect_accounts(all_accounts);
        }
    }

    /// Normalizes the bundle into a valid, canonical form.
    ///
    /// Effects:
    /// - Deduplicates committed accounts within each intent by pubkey (stable; first occurrence wins).
    /// - Resolves overlap between `Commit` and `CommitAndUndelegate`:
    ///   any account present in both will be removed from `Commit` and kept in `CommitAndUndelegate`.
    /// - If `Commit` becomes empty after overlap removal, it is removed from the bundle, and any
    ///   post-commit handlers from the commit intent are merged into the commit-side handlers
    ///   of `CommitAndUndelegate`.
    fn normalize(&mut self) {
        // Remove duplicates inside individual intents
        if let Some(ref mut value) = self.commit_intent {
            value.dedup();
        }
        let cau = self.commit_and_undelegate_intent.as_mut().map(|value| {
            let seen = value.dedup();
            (seen, value)
        });

        // Remove cross intent duplicates
        let (mut commit, cau, cau_pubkeys) = match (self.commit_intent.take(), cau) {
            (Some(commit), Some((cau_pubkeys, cau))) => (commit, cau, cau_pubkeys),
            // In case only one Intent exist we can exit
            _ => return,
        };

        // If accounts in CommitAndUndelegate and Commit intents overlap
        // we keep them only in CommitAndUndelegate Intent and remove from Commit
        commit
            .committed_accounts_mut()
            .retain(|el| !cau_pubkeys.contains(el.key));

        // edge case
        if commit.committed_accounts().is_empty() {
            // if after dedup Commit intent becomes empty
            // 1. remove it from bundle
            // 2. if it has action - move them in CommitAndUndelegate intent
            cau.commit_type.merge(commit);
        } else {
            // set deduped intent
            self.commit_intent = Some(commit);
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
