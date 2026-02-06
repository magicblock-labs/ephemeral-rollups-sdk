use crate::intent_bundle::args::{
    ActionArgs, BaseActionArgs, CommitAndUndelegateArgs, CommitTypeArgs, MagicIntentBundleArgs,
    ShortAccountMeta, UndelegateTypeArgs,
};
use crate::intent_bundle::no_vec::NoVec;
use pinocchio::cpi::MAX_STATIC_CPI_ACCOUNTS;
use pinocchio::AccountView;
use solana_address::Address;

use super::MAX_ACTIONS_NUM;

// TODO: switch to err
const EXPECTED_KEY_MSG: &str = "Key expected to exist!";

/// Intent to be scheduled for execution on the base layer.
///
/// This enum represents the different types of operations that can be bundled
/// and executed through the Magic program.
#[allow(clippy::large_enum_variant)]
pub enum MagicIntent<'args> {
    /// Standalone actions to execute on base layer without commit/undelegate semantics.
    StandaloneActions(NoVec<CallHandler<'args>, MAX_ACTIONS_NUM>),
    /// Commit accounts to base layer, optionally with post-commit actions.
    Commit(CommitIntent<'args>),
    /// Commit accounts and undelegate them, optionally with post-commit and post-undelegate actions.
    CommitAndUndelegate(CommitAndUndelegateIntent<'args>),
}

/// Bundle of Intents
///
/// Note: if `CommitIntent` & `CommitAndUndelegateIntent` has an account overlap
/// they will be undelegated
///
/// Intents assumed to be independent and self-sufficient,
/// hence order in which they were inserted doesn't matter
#[derive(Default)]
pub(super) struct MagicIntentBundle<'args> {
    standalone_actions: NoVec<CallHandler<'args>, MAX_ACTIONS_NUM>,
    commit_intent: Option<CommitIntent<'args>>,
    commit_and_undelegate_intent: Option<CommitAndUndelegateIntent<'args>>,
}

impl<'args> MagicIntentBundle<'args> {
    /// Inserts an intent into the bundle, merging with any existing intent of the same category.
    pub(super) fn add_intent(&mut self, intent: MagicIntent<'args>) {
        match intent {
            MagicIntent::StandaloneActions(value) => self.standalone_actions.extend(value),
            MagicIntent::Commit(value) => {
                if let Some(ref mut existing) = self.commit_intent {
                    existing.merge(value);
                } else {
                    self.commit_intent = Some(value);
                }
            }
            MagicIntent::CommitAndUndelegate(value) => {
                if let Some(ref mut existing) = self.commit_and_undelegate_intent {
                    existing.merge(value);
                } else {
                    self.commit_and_undelegate_intent = Some(value);
                }
            }
        }
    }

    /// Consumes the bundle and encodes it into `MagicIntentBundleArgs` using an indices map.
    ///
    /// `indices_map` is a natural map: `indices_map[i]` is the address at index `i`.
    pub(super) fn into_args(self, indices_map: &[Address]) -> MagicIntentBundleArgs<'args> {
        let commit = self.commit_intent.map(|c| c.into_args(indices_map));
        let commit_and_undelegate = self
            .commit_and_undelegate_intent
            .map(|c| c.into_args(indices_map));
        let standalone_actions = self
            .standalone_actions
            .into_iter()
            .map(|ch| ch.into_args(indices_map))
            .collect::<NoVec<BaseActionArgs<'args>, MAX_ACTIONS_NUM>>();

        MagicIntentBundleArgs {
            commit,
            commit_and_undelegate,
            standalone_actions,
        }
    }

    /// Collects all accounts referenced by intents in this bundle.
    pub(super) fn collect_unique_accounts(
        &self,
        unique_accounts: &mut NoVec<AccountView, MAX_STATIC_CPI_ACCOUNTS>,
    ) {
        for el in &self.standalone_actions {
            el.collect_unique_accounts(unique_accounts);
        }
        if let Some(commit) = &self.commit_intent {
            commit.collect_unique_accounts(unique_accounts);
        }
        if let Some(cau) = &self.commit_and_undelegate_intent {
            cau.collect_unique_accounts(unique_accounts);
        }
    }

    /// Normalizes the bundle into a valid, canonical form.
    ///
    /// Effects:
    /// - Deduplicates committed accounts within each intent by address (stable; first occurrence wins).
    /// - Resolves overlap between `Commit` and `CommitAndUndelegate`:
    ///   any account present in both will be removed from `Commit` and kept in `CommitAndUndelegate`.
    /// - If `Commit` becomes empty after overlap removal, it is removed from the bundle, and any
    ///   post-commit actions from the commit intent are merged into the commit-side actions
    ///   of `CommitAndUndelegate`.
    pub(super) fn normalize(&mut self) {
        let cau_seen = self
            .commit_and_undelegate_intent
            .as_mut()
            .map(|cau| cau.dedup());
        let Some(mut commit) = self.commit_intent.take() else {
            return; // No commit intent, nothing more to normalize
        };

        let mut seen = cau_seen.unwrap_or_default();
        commit.dedup(&mut seen);

        // If commit lost all its accounts to CAU overlap, move its actions
        // into CAU's post-commit actions and drop the empty commit intent.
        if commit.accounts.is_empty() {
            if let Some(ref mut cau) = self.commit_and_undelegate_intent {
                for action in commit.actions {
                    cau.post_commit_actions.push(action);
                }
            }
            // commit intent is not put back — it's effectively removed
        } else {
            self.commit_intent = Some(commit);
        }
    }
}

// ---------------------------------------------------------------------------
// Intent Types
// ---------------------------------------------------------------------------

#[derive(Clone)]
pub struct CallHandler<'args> {
    pub destination_program: Address,
    pub escrow_authority: AccountView,
    pub args: ActionArgs<'args>,
    pub compute_units: u32,
    accounts: NoVec<ShortAccountMeta, MAX_ACTIONS_NUM>,
}

impl<'args> CallHandler<'args> {
    pub fn new(
        destination_program: Address,
        escrow_authority: AccountView,
        args: ActionArgs<'args>,
        compute_units: u32,
    ) -> Self {
        Self {
            args,
            compute_units,
            escrow_authority,
            destination_program,
            accounts: NoVec::default(),
        }
    }

    pub fn add_accounts_slice(&mut self, accounts: &[ShortAccountMeta]) {
        self.accounts.append_slice(accounts);
    }

    pub fn add_accounts<const N: usize>(&mut self, accounts: [ShortAccountMeta; N]) {
        self.accounts.append(accounts);
    }

    pub(super) fn collect_unique_accounts(
        &self,
        container: &mut NoVec<AccountView, MAX_STATIC_CPI_ACCOUNTS>,
    ) {
        if !container.contains(&self.escrow_authority) {
            container.push(self.escrow_authority.clone());
        }
    }

    pub(crate) fn into_args(self, indices_map: &[Address]) -> BaseActionArgs<'args> {
        let escrow_authority_index =
            get_index(indices_map, self.escrow_authority.address()).expect(EXPECTED_KEY_MSG);
        BaseActionArgs {
            args: self.args,
            compute_units: self.compute_units,
            destination_program: self.destination_program,
            escrow_authority: escrow_authority_index,
            accounts: self.accounts,
        }
    }
}

pub struct CommitIntent<'args> {
    pub(super) accounts: NoVec<AccountView, MAX_STATIC_CPI_ACCOUNTS>,
    pub(super) actions: NoVec<CallHandler<'args>, MAX_ACTIONS_NUM>,
}

impl<'args> CommitIntent<'args> {
    /// Deduplicates committed accounts by address. Accounts whose address is
    /// already in `seen` are removed. Newly seen addresses are added to `seen`.
    fn dedup(&mut self, seen: &mut NoVec<Address, MAX_STATIC_CPI_ACCOUNTS>) {
        self.accounts.retain(|el| {
            let addr = el.address();
            if seen.contains(addr) {
                false
            } else {
                seen.push(addr.clone());
                true
            }
        });
    }

    /// Merges another CommitIntent into this one. Only inserts accounts
    /// whose address is not already present (dedup on merge).
    fn merge(&mut self, other: Self) {
        for account in other.accounts {
            if !self
                .accounts
                .as_slice()
                .iter()
                .any(|a| a.address() == account.address())
            {
                self.accounts.push(account);
            }
        }
        for action in other.actions {
            self.actions.push(action);
        }
    }

    fn collect_unique_accounts(&self, container: &mut NoVec<AccountView, MAX_STATIC_CPI_ACCOUNTS>) {
        for el in self.accounts.iter() {
            if !container.contains(el) {
                container.push(el.clone());
            }
        }
        for el in self.actions.iter() {
            el.collect_unique_accounts(container);
        }
    }

    fn into_args(self, indices_map: &[Address]) -> CommitTypeArgs<'args> {
        let mut indices = NoVec::<u8, MAX_STATIC_CPI_ACCOUNTS>::new();
        for account in self.accounts {
            let idx = get_index(indices_map, account.address()).expect(EXPECTED_KEY_MSG);
            indices.push(idx);
        }

        if self.actions.is_empty() {
            CommitTypeArgs::Standalone(indices)
        } else {
            let mut base_actions = NoVec::<_, MAX_ACTIONS_NUM>::new();
            for handler in self.actions {
                base_actions.push(handler.into_args(indices_map));
            }
            CommitTypeArgs::WithBaseActions {
                committed_accounts: indices,
                base_actions,
            }
        }
    }
}

pub struct CommitAndUndelegateIntent<'args> {
    pub(super) accounts: NoVec<AccountView, MAX_STATIC_CPI_ACCOUNTS>,
    pub(super) post_commit_actions: NoVec<CallHandler<'args>, MAX_ACTIONS_NUM>,
    pub(super) post_undelegate_actions: NoVec<CallHandler<'args>, MAX_ACTIONS_NUM>,
}

impl<'args> CommitAndUndelegateIntent<'args> {
    /// Deduplicates committed accounts by address and returns the set of
    /// unique addresses (for cross-intent overlap detection).
    fn dedup(&mut self) -> NoVec<Address, MAX_STATIC_CPI_ACCOUNTS> {
        let mut seen = NoVec::<Address, MAX_STATIC_CPI_ACCOUNTS>::new();
        self.accounts.retain(|el| {
            let addr = el.address();
            if seen.contains(addr) {
                false
            } else {
                seen.push(addr.clone());
                true
            }
        });
        seen
    }

    /// Merges another CommitAndUndelegateIntent into this one. Only inserts
    /// accounts whose address is not already present (dedup on merge).
    fn merge(&mut self, other: Self) {
        for account in other.accounts {
            if !self
                .accounts
                .as_slice()
                .iter()
                .any(|a| a.address() == account.address())
            {
                self.accounts.push(account);
            }
        }
        for action in other.post_commit_actions {
            self.post_commit_actions.push(action);
        }
        for action in other.post_undelegate_actions {
            self.post_undelegate_actions.push(action);
        }
    }

    fn collect_unique_accounts(&self, container: &mut NoVec<AccountView, MAX_STATIC_CPI_ACCOUNTS>) {
        for el in self.accounts.iter() {
            if !container.contains(el) {
                container.push(el.clone());
            }
        }
        for el in self.post_commit_actions.iter() {
            el.collect_unique_accounts(container);
        }
        for el in self.post_undelegate_actions.iter() {
            el.collect_unique_accounts(container);
        }
    }

    fn into_args(self, indices_map: &[Address]) -> CommitAndUndelegateArgs<'args> {
        // Build account indices
        let mut indices = NoVec::<u8, MAX_STATIC_CPI_ACCOUNTS>::new();
        for account in self.accounts {
            let idx = get_index(indices_map, account.address()).expect(EXPECTED_KEY_MSG);
            indices.push(idx);
        }

        // Build commit type
        let commit_type = if self.post_commit_actions.is_empty() {
            CommitTypeArgs::Standalone(indices)
        } else {
            let mut base_actions = NoVec::<_, MAX_ACTIONS_NUM>::new();
            for handler in self.post_commit_actions {
                base_actions.push(handler.into_args(indices_map));
            }
            CommitTypeArgs::WithBaseActions {
                committed_accounts: indices,
                base_actions,
            }
        };

        // Build undelegate type
        let undelegate_type = if self.post_undelegate_actions.is_empty() {
            UndelegateTypeArgs::Standalone
        } else {
            let mut base_actions = NoVec::<_, MAX_ACTIONS_NUM>::new();
            for handler in self.post_undelegate_actions {
                base_actions.push(handler.into_args(indices_map));
            }
            UndelegateTypeArgs::WithBaseActions { base_actions }
        };

        CommitAndUndelegateArgs {
            commit_type,
            undelegate_type,
        }
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Gets the index of a pubkey in the deduplicated pubkey list.
/// Returns None if the pubkey is not found.
fn get_index(pubkeys: &[Address], needle: &Address) -> Option<u8> {
    pubkeys.iter().position(|k| k == needle).map(|i| i as u8)
}
