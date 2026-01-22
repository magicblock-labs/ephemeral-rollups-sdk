#![allow(deprecated)]

pub use crate::ephem::deprecated::v0::{
    commit_accounts, commit_and_undelegate_accounts, create_schedule_commit_ix,
};
use crate::ephem::deprecated::v1::utils;
pub use crate::ephem::deprecated::v1::{
    CallHandler, CommitAndUndelegate, CommitType, MagicAction, MagicInstructionBuilder,
    UndelegateType,
};
use crate::solana_compat::solana::{
    invoke, AccountInfo, AccountMeta, Instruction, ProgramResult, Pubkey,
};
use magicblock_magic_program_api::args::MagicIntentBundleArgs;
use magicblock_magic_program_api::instruction::MagicBlockInstruction;
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
        self.intent_bundle.add_intent(MagicBaseIntent::Commit(commit));
        self
    }

    /// Adds (or merges) a `CommitAndUndelegate` intent into the bundle.
    pub fn add_commit_and_undelegate_intent(mut self, value: CommitAndUndelegate<'info>) -> Self {
        self.intent_bundle
            .add_intent(MagicBaseIntent::CommitAndUndelegate(value));
        self
    }

    /// Adds standalone base-layer actions to be executed without any commit/undelegate semantics.
    pub fn add_base_actions_intent(
        mut self,
        actions: impl IntoIterator<Item = CallHandler<'info>>,
    ) -> Self {
        self.intent_bundle
            .add_intent(MagicBaseIntent::BaseActions(actions.into_iter().collect()));
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

        // Collect all accounts used by the bundle, then dedup them + create index map.
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
        let commit = self.commit_intent.map(|c| c.into_args(indices_map));
        let commit_and_undelegate = self
            .commit_and_undelegate_intent
            .map(|c| c.into_args(indices_map));
        let standalone_actions = self
            .standalone_actions
            .into_iter()
            .map(|ch| ch.into_args(indices_map))
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
        // Only proceed if both intents exist; otherwise no cross-intent dedup needed
        let (mut commit, cau, cau_pubkeys) = match (self.commit_intent.take(), cau) {
            (Some(commit), Some((cau_pubkeys, cau))) => (commit, cau, cau_pubkeys),
            // In case only one Intent exists, put commit_intent back if it was taken
            (Some(commit), None) => {
                self.commit_intent = Some(commit);
                return;
            }
            // No commit_intent or neither intent exists - nothing to restore
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::solana_compat::solana::{AccountInfo, Pubkey};
    use magicblock_magic_program_api::args::ActionArgs;
    use std::cell::RefCell;
    use std::rc::Rc;

    /// Helper to create a mock AccountInfo for testing
    fn create_mock_account_info<'a>(
        key: &'a Pubkey,
        lamports: &'a mut u64,
        data: &'a mut [u8],
        owner: &'a Pubkey,
        is_signer: bool,
        is_writable: bool,
    ) -> AccountInfo<'a> {
        AccountInfo {
            key,
            is_signer,
            is_writable,
            lamports: Rc::new(RefCell::new(lamports)),
            data: Rc::new(RefCell::new(data)),
            owner,
            executable: false,
            rent_epoch: 0,
        }
    }

    /// Helper struct to hold account data for tests
    #[allow(dead_code)]
    struct TestAccount {
        key: Pubkey,
        lamports: u64,
        data: Vec<u8>,
        owner: Pubkey,
    }

    impl TestAccount {
        fn new() -> Self {
            Self {
                key: Pubkey::new_unique(),
                lamports: 1_000_000,
                data: vec![0u8; 32],
                owner: Pubkey::new_unique(),
            }
        }

        fn with_key(key: Pubkey) -> Self {
            Self {
                key,
                lamports: 1_000_000,
                data: vec![0u8; 32],
                owner: Pubkey::new_unique(),
            }
        }
    }

    /// Helper to create a CallHandler for testing
    fn create_test_call_handler(escrow_authority: AccountInfo) -> CallHandler {
        CallHandler {
            args: ActionArgs::new(vec![1, 2, 3]),
            compute_units: 100_000,
            escrow_authority,
            destination_program: Pubkey::new_unique(),
            accounts: vec![],
        }
    }

    // ----- CommitIntentBuilder Tests -----

    #[test]
    fn test_commit_intent_builder_standalone() {
        let mut acc1 = TestAccount::new();
        let mut acc2 = TestAccount::new();
        let owner = Pubkey::new_unique();

        let info1 = create_mock_account_info(
            &acc1.key,
            &mut acc1.lamports,
            &mut acc1.data,
            &owner,
            false,
            true,
        );
        let info2 = create_mock_account_info(
            &acc2.key,
            &mut acc2.lamports,
            &mut acc2.data,
            &owner,
            false,
            true,
        );

        let accounts = vec![info1, info2];
        let commit = CommitIntentBuilder::new(&accounts).build();

        match commit {
            CommitType::Standalone(accs) => {
                assert_eq!(accs.len(), 2);
                assert_eq!(*accs[0].key, acc1.key);
                assert_eq!(*accs[1].key, acc2.key);
            }
            CommitType::WithHandler { .. } => panic!("Expected Standalone variant"),
        }
    }

    #[test]
    fn test_commit_intent_builder_with_handler() {
        let mut acc1 = TestAccount::new();
        let mut escrow_acc = TestAccount::new();
        let owner = Pubkey::new_unique();

        let info1 = create_mock_account_info(
            &acc1.key,
            &mut acc1.lamports,
            &mut acc1.data,
            &owner,
            false,
            true,
        );
        let escrow_info = create_mock_account_info(
            &escrow_acc.key,
            &mut escrow_acc.lamports,
            &mut escrow_acc.data,
            &owner,
            true,
            false,
        );

        let accounts = vec![info1];
        let handler = create_test_call_handler(escrow_info);
        let commit = CommitIntentBuilder::new(&accounts)
            .add_post_commit_actions([handler])
            .build();

        match commit {
            CommitType::WithHandler {
                commited_accounts,
                call_handlers,
            } => {
                assert_eq!(commited_accounts.len(), 1);
                assert_eq!(call_handlers.len(), 1);
            }
            CommitType::Standalone(_) => panic!("Expected WithHandler variant"),
        }
    }

    // ----- CommitAndUndelegateIntentBuilder Tests -----

    #[test]
    fn test_commit_and_undelegate_builder_standalone() {
        let mut acc1 = TestAccount::new();
        let owner = Pubkey::new_unique();

        let info1 = create_mock_account_info(
            &acc1.key,
            &mut acc1.lamports,
            &mut acc1.data,
            &owner,
            false,
            true,
        );

        let accounts = vec![info1];
        let cau = CommitAndUndelegateIntentBuilder::new(&accounts).build();

        match (&cau.commit_type, &cau.undelegate_type) {
            (CommitType::Standalone(accs), UndelegateType::Standalone) => {
                assert_eq!(accs.len(), 1);
            }
            _ => panic!("Expected Standalone variants"),
        }
    }

    #[test]
    fn test_commit_and_undelegate_builder_with_actions() {
        let mut acc1 = TestAccount::new();
        let mut escrow_acc1 = TestAccount::new();
        let mut escrow_acc2 = TestAccount::new();
        let owner = Pubkey::new_unique();

        let info1 = create_mock_account_info(
            &acc1.key,
            &mut acc1.lamports,
            &mut acc1.data,
            &owner,
            false,
            true,
        );
        let escrow_info1 = create_mock_account_info(
            &escrow_acc1.key,
            &mut escrow_acc1.lamports,
            &mut escrow_acc1.data,
            &owner,
            true,
            false,
        );
        let escrow_info2 = create_mock_account_info(
            &escrow_acc2.key,
            &mut escrow_acc2.lamports,
            &mut escrow_acc2.data,
            &owner,
            true,
            false,
        );

        let accounts = vec![info1];
        let post_commit_handler = create_test_call_handler(escrow_info1);
        let post_undelegate_handler = create_test_call_handler(escrow_info2);

        let cau = CommitAndUndelegateIntentBuilder::new(&accounts)
            .add_post_commit_actions([post_commit_handler])
            .add_post_undelegate_actions([post_undelegate_handler])
            .build();

        match (&cau.commit_type, &cau.undelegate_type) {
            (
                CommitType::WithHandler { call_handlers, .. },
                UndelegateType::WithHandler(undelegate_handlers),
            ) => {
                assert_eq!(call_handlers.len(), 1);
                assert_eq!(undelegate_handlers.len(), 1);
            }
            _ => panic!("Expected WithHandler variants"),
        }
    }

    // ----- MagicIntentBundle Normalization Tests -----

    #[test]
    fn test_bundle_dedup_within_commit_intent() {
        let shared_key = Pubkey::new_unique();
        let mut acc1 = TestAccount::with_key(shared_key);
        let mut acc2 = TestAccount::with_key(shared_key); // duplicate key
        let mut acc3 = TestAccount::new();
        let owner = Pubkey::new_unique();

        let info1 = create_mock_account_info(
            &acc1.key,
            &mut acc1.lamports,
            &mut acc1.data,
            &owner,
            false,
            true,
        );
        let info2 = create_mock_account_info(
            &acc2.key,
            &mut acc2.lamports,
            &mut acc2.data,
            &owner,
            false,
            true,
        );
        let info3 = create_mock_account_info(
            &acc3.key,
            &mut acc3.lamports,
            &mut acc3.data,
            &owner,
            false,
            true,
        );

        let mut bundle = MagicIntentBundle::default();
        bundle.add_intent(MagicBaseIntent::Commit(CommitType::Standalone(vec![
            info1, info2, info3,
        ])));

        bundle.normalize();

        let commit = bundle.commit_intent.expect("commit should exist");
        let accounts = commit.committed_accounts();
        assert_eq!(accounts.len(), 2, "Duplicates should be removed");
    }

    #[test]
    fn test_bundle_cross_intent_overlap_resolution() {
        // Accounts in both Commit and CommitAndUndelegate should only be in CommitAndUndelegate
        let shared_key = Pubkey::new_unique();
        let mut shared_acc1 = TestAccount::with_key(shared_key);
        let mut shared_acc2 = TestAccount::with_key(shared_key);
        let mut unique_acc = TestAccount::new();
        let owner = Pubkey::new_unique();

        let shared_info1 = create_mock_account_info(
            &shared_acc1.key,
            &mut shared_acc1.lamports,
            &mut shared_acc1.data,
            &owner,
            false,
            true,
        );
        let shared_info2 = create_mock_account_info(
            &shared_acc2.key,
            &mut shared_acc2.lamports,
            &mut shared_acc2.data,
            &owner,
            false,
            true,
        );
        let unique_info = create_mock_account_info(
            &unique_acc.key,
            &mut unique_acc.lamports,
            &mut unique_acc.data,
            &owner,
            false,
            true,
        );

        let mut bundle = MagicIntentBundle::default();

        // Add shared account to Commit intent along with a unique one
        bundle.add_intent(MagicBaseIntent::Commit(CommitType::Standalone(vec![
            shared_info1,
            unique_info,
        ])));

        // Add shared account to CommitAndUndelegate intent
        bundle.add_intent(MagicBaseIntent::CommitAndUndelegate(CommitAndUndelegate {
            commit_type: CommitType::Standalone(vec![shared_info2]),
            undelegate_type: UndelegateType::Standalone,
        }));

        bundle.normalize();

        // Commit intent should only have the unique account
        let commit = bundle.commit_intent.expect("commit should exist");
        assert_eq!(commit.committed_accounts().len(), 1);
        assert_eq!(*commit.committed_accounts()[0].key, unique_acc.key);

        // CommitAndUndelegate should have the shared account
        let cau = bundle
            .commit_and_undelegate_intent
            .expect("cau should exist");
        assert_eq!(cau.commit_type.committed_accounts().len(), 1);
        assert_eq!(*cau.commit_type.committed_accounts()[0].key, shared_key);
    }

    #[test]
    fn test_bundle_commit_becomes_empty_after_overlap() {
        // When all Commit accounts are in CommitAndUndelegate, Commit is removed
        // and its handlers are merged into CommitAndUndelegate
        let shared_key = Pubkey::new_unique();
        let mut shared_acc1 = TestAccount::with_key(shared_key);
        let mut shared_acc2 = TestAccount::with_key(shared_key);
        let mut escrow_acc = TestAccount::new();
        let owner = Pubkey::new_unique();

        let shared_info1 = create_mock_account_info(
            &shared_acc1.key,
            &mut shared_acc1.lamports,
            &mut shared_acc1.data,
            &owner,
            false,
            true,
        );
        let shared_info2 = create_mock_account_info(
            &shared_acc2.key,
            &mut shared_acc2.lamports,
            &mut shared_acc2.data,
            &owner,
            false,
            true,
        );
        let escrow_info = create_mock_account_info(
            &escrow_acc.key,
            &mut escrow_acc.lamports,
            &mut escrow_acc.data,
            &owner,
            true,
            false,
        );

        let handler = create_test_call_handler(escrow_info);

        let mut bundle = MagicIntentBundle::default();

        // Add shared account to Commit intent with a handler
        bundle.add_intent(MagicBaseIntent::Commit(CommitType::WithHandler {
            commited_accounts: vec![shared_info1],
            call_handlers: vec![handler],
        }));

        // Add same account to CommitAndUndelegate
        bundle.add_intent(MagicBaseIntent::CommitAndUndelegate(CommitAndUndelegate {
            commit_type: CommitType::Standalone(vec![shared_info2]),
            undelegate_type: UndelegateType::Standalone,
        }));

        bundle.normalize();

        // Commit intent should be removed (was empty after overlap resolution)
        assert!(
            bundle.commit_intent.is_none(),
            "commit should be removed when empty"
        );

        // CommitAndUndelegate should have the handler merged
        let cau = bundle
            .commit_and_undelegate_intent
            .expect("cau should exist");
        match &cau.commit_type {
            CommitType::WithHandler { call_handlers, .. } => {
                assert_eq!(call_handlers.len(), 1, "Handler should be merged");
            }
            CommitType::Standalone(_) => panic!("Expected WithHandler after merge"),
        }
    }

    // ----- MagicIntentBundle Intent Merging Tests -----

    #[test]
    fn test_bundle_merge_multiple_commit_intents() {
        let mut acc1 = TestAccount::new();
        let mut acc2 = TestAccount::new();
        let owner = Pubkey::new_unique();

        let info1 = create_mock_account_info(
            &acc1.key,
            &mut acc1.lamports,
            &mut acc1.data,
            &owner,
            false,
            true,
        );
        let info2 = create_mock_account_info(
            &acc2.key,
            &mut acc2.lamports,
            &mut acc2.data,
            &owner,
            false,
            true,
        );

        let mut bundle = MagicIntentBundle::default();

        // Add first commit intent
        bundle.add_intent(MagicBaseIntent::Commit(CommitType::Standalone(vec![info1])));

        // Add second commit intent - should merge
        bundle.add_intent(MagicBaseIntent::Commit(CommitType::Standalone(vec![info2])));

        let commit = bundle.commit_intent.expect("commit should exist");
        assert_eq!(
            commit.committed_accounts().len(),
            2,
            "Accounts should be merged"
        );
    }

    #[test]
    fn test_bundle_merge_base_actions() {
        let mut escrow_acc1 = TestAccount::new();
        let mut escrow_acc2 = TestAccount::new();
        let owner = Pubkey::new_unique();

        let escrow_info1 = create_mock_account_info(
            &escrow_acc1.key,
            &mut escrow_acc1.lamports,
            &mut escrow_acc1.data,
            &owner,
            true,
            false,
        );
        let escrow_info2 = create_mock_account_info(
            &escrow_acc2.key,
            &mut escrow_acc2.lamports,
            &mut escrow_acc2.data,
            &owner,
            true,
            false,
        );

        let handler1 = create_test_call_handler(escrow_info1);
        let handler2 = create_test_call_handler(escrow_info2);

        let mut bundle = MagicIntentBundle::default();

        bundle.add_intent(MagicBaseIntent::BaseActions(vec![handler1]));
        bundle.add_intent(MagicBaseIntent::BaseActions(vec![handler2]));

        assert_eq!(bundle.standalone_actions.len(), 2);
    }

    // ----- MagicIntentBundleBuilder Tests -----

    #[test]
    fn test_builder_creates_instruction_with_commit_only() {
        let mut payer = TestAccount::new();
        let mut magic_ctx = TestAccount::new();
        let mut magic_prog = TestAccount::new();
        let mut acc1 = TestAccount::new();
        let owner = Pubkey::new_unique();

        let payer_info = create_mock_account_info(
            &payer.key,
            &mut payer.lamports,
            &mut payer.data,
            &owner,
            true,
            true,
        );
        let ctx_info = create_mock_account_info(
            &magic_ctx.key,
            &mut magic_ctx.lamports,
            &mut magic_ctx.data,
            &owner,
            false,
            true,
        );
        let prog_info = create_mock_account_info(
            &magic_prog.key,
            &mut magic_prog.lamports,
            &mut magic_prog.data,
            &owner,
            false,
            false,
        );
        let acc1_info = create_mock_account_info(
            &acc1.key,
            &mut acc1.lamports,
            &mut acc1.data,
            &owner,
            false,
            true,
        );

        let commit = CommitType::Standalone(vec![acc1_info]);

        let (accounts, ix) = MagicIntentBundleBuilder::new(payer_info, ctx_info, prog_info)
            .add_commit_intent(commit)
            .build();

        // Verify accounts: payer, context, acc1
        assert_eq!(accounts.len(), 3);
        assert_eq!(*accounts[0].key, payer.key);
        assert_eq!(*accounts[1].key, magic_ctx.key);
        assert_eq!(*accounts[2].key, acc1.key);

        // Verify instruction program
        assert_eq!(ix.program_id, magic_prog.key);
    }

    #[test]
    fn test_builder_deduplicates_accounts() {
        let mut payer = TestAccount::new();
        let mut magic_ctx = TestAccount::new();
        let mut magic_prog = TestAccount::new();

        // Same key used multiple times
        let shared_key = Pubkey::new_unique();
        let mut acc1 = TestAccount::with_key(shared_key);
        let mut acc2 = TestAccount::with_key(shared_key);
        let owner = Pubkey::new_unique();

        let payer_info = create_mock_account_info(
            &payer.key,
            &mut payer.lamports,
            &mut payer.data,
            &owner,
            true,
            true,
        );
        let ctx_info = create_mock_account_info(
            &magic_ctx.key,
            &mut magic_ctx.lamports,
            &mut magic_ctx.data,
            &owner,
            false,
            true,
        );
        let prog_info = create_mock_account_info(
            &magic_prog.key,
            &mut magic_prog.lamports,
            &mut magic_prog.data,
            &owner,
            false,
            false,
        );
        let acc1_info = create_mock_account_info(
            &acc1.key,
            &mut acc1.lamports,
            &mut acc1.data,
            &owner,
            false,
            true,
        );
        let acc2_info = create_mock_account_info(
            &acc2.key,
            &mut acc2.lamports,
            &mut acc2.data,
            &owner,
            false,
            true,
        );

        let commit = CommitType::Standalone(vec![acc1_info, acc2_info]);

        let (accounts, _ix) = MagicIntentBundleBuilder::new(payer_info, ctx_info, prog_info)
            .add_commit_intent(commit)
            .build();

        // Should be: payer, context, shared_account (deduplicated)
        assert_eq!(accounts.len(), 3, "Duplicate accounts should be removed");
    }

    #[test]
    fn test_builder_with_all_intent_types() {
        let mut payer = TestAccount::new();
        let mut magic_ctx = TestAccount::new();
        let mut magic_prog = TestAccount::new();
        let mut commit_acc = TestAccount::new();
        let mut cau_acc = TestAccount::new();
        let mut escrow_acc = TestAccount::new();
        let owner = Pubkey::new_unique();

        let payer_info = create_mock_account_info(
            &payer.key,
            &mut payer.lamports,
            &mut payer.data,
            &owner,
            true,
            true,
        );
        let ctx_info = create_mock_account_info(
            &magic_ctx.key,
            &mut magic_ctx.lamports,
            &mut magic_ctx.data,
            &owner,
            false,
            true,
        );
        let prog_info = create_mock_account_info(
            &magic_prog.key,
            &mut magic_prog.lamports,
            &mut magic_prog.data,
            &owner,
            false,
            false,
        );
        let commit_info = create_mock_account_info(
            &commit_acc.key,
            &mut commit_acc.lamports,
            &mut commit_acc.data,
            &owner,
            false,
            true,
        );
        let cau_info = create_mock_account_info(
            &cau_acc.key,
            &mut cau_acc.lamports,
            &mut cau_acc.data,
            &owner,
            false,
            true,
        );
        let escrow_info = create_mock_account_info(
            &escrow_acc.key,
            &mut escrow_acc.lamports,
            &mut escrow_acc.data,
            &owner,
            true,
            false,
        );

        let commit = CommitType::Standalone(vec![commit_info]);
        let cau = CommitAndUndelegate {
            commit_type: CommitType::Standalone(vec![cau_info]),
            undelegate_type: UndelegateType::Standalone,
        };
        let handler = create_test_call_handler(escrow_info);

        let (accounts, ix) = MagicIntentBundleBuilder::new(payer_info, ctx_info, prog_info)
            .add_commit_intent(commit)
            .add_commit_and_undelegate_intent(cau)
            .add_base_actions_intent([handler])
            .build();

        // Should have: payer, context, commit_acc, cau_acc, escrow_acc
        assert_eq!(accounts.len(), 5);

        // Verify instruction data contains ScheduleIntentBundle
        assert!(!ix.data.is_empty());
    }

    #[test]
    fn test_builder_fluent_api() {
        let mut payer = TestAccount::new();
        let mut magic_ctx = TestAccount::new();
        let mut magic_prog = TestAccount::new();
        let mut acc1 = TestAccount::new();
        let owner = Pubkey::new_unique();

        let payer_info = create_mock_account_info(
            &payer.key,
            &mut payer.lamports,
            &mut payer.data,
            &owner,
            true,
            true,
        );
        let ctx_info = create_mock_account_info(
            &magic_ctx.key,
            &mut magic_ctx.lamports,
            &mut magic_ctx.data,
            &owner,
            false,
            true,
        );
        let prog_info = create_mock_account_info(
            &magic_prog.key,
            &mut magic_prog.lamports,
            &mut magic_prog.data,
            &owner,
            false,
            false,
        );
        let acc1_info = create_mock_account_info(
            &acc1.key,
            &mut acc1.lamports,
            &mut acc1.data,
            &owner,
            false,
            true,
        );

        let accounts_slice = vec![acc1_info];

        // Test that builder methods chain properly
        let (accounts, _ix) = MagicIntentBundleBuilder::new(payer_info, ctx_info, prog_info)
            .add_intent(MagicBaseIntent::Commit(
                CommitIntentBuilder::new(&accounts_slice).build(),
            ))
            .build();

        assert_eq!(accounts.len(), 3);
    }

    // ----- Account Collection Tests -----

    #[test]
    fn test_collect_accounts_from_call_handler() {
        let mut escrow_acc = TestAccount::new();
        let owner = Pubkey::new_unique();

        let escrow_info = create_mock_account_info(
            &escrow_acc.key,
            &mut escrow_acc.lamports,
            &mut escrow_acc.data,
            &owner,
            true,
            false,
        );

        let handler = create_test_call_handler(escrow_info);
        let mut container = Vec::new();
        handler.collect_accounts(&mut container);

        assert_eq!(container.len(), 1);
        assert_eq!(*container[0].key, escrow_acc.key);
    }

    #[test]
    fn test_collect_accounts_from_commit_with_handler() {
        let mut acc1 = TestAccount::new();
        let mut escrow_acc = TestAccount::new();
        let owner = Pubkey::new_unique();

        let acc1_info = create_mock_account_info(
            &acc1.key,
            &mut acc1.lamports,
            &mut acc1.data,
            &owner,
            false,
            true,
        );
        let escrow_info = create_mock_account_info(
            &escrow_acc.key,
            &mut escrow_acc.lamports,
            &mut escrow_acc.data,
            &owner,
            true,
            false,
        );

        let handler = create_test_call_handler(escrow_info);
        let commit = CommitType::WithHandler {
            commited_accounts: vec![acc1_info],
            call_handlers: vec![handler],
        };

        let mut container = Vec::new();
        commit.collect_accounts(&mut container);

        // Should have both the committed account and escrow from handler
        assert_eq!(container.len(), 2);
    }
}
