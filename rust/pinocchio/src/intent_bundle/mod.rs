use crate::intent_bundle::args::{
    ActionArgs, BaseActionArgs, CommitAndUndelegateArgs, CommitTypeArgs, MagicIntentBundleArgs,
    ShortAccountMeta, UndelegateTypeArgs,
};
use crate::intent_bundle::no_vec::NoVec;
use pinocchio::cpi::invoke_with_bounds;
use pinocchio::error::ProgramError;
use pinocchio::instruction::{InstructionAccount, InstructionView};
use pinocchio::{AccountView, ProgramResult};
use solana_address::Address;

mod args;
mod no_vec;
mod types;

const MAX_ACTIONS_NUM: usize = 10u8 as usize;
const MAX_COMMITTED_ACCOUNTS_NUM: usize = 64u8 as usize;
const MAX_ACCOUNTS: usize = pinocchio::cpi::MAX_CPI_ACCOUNTS;

// TODO: switch to err
const EXPECTED_KEY_MSG: &str = "Key expected to exist!";

/// Intent to be scheduled for execution on the base layer.
///
/// This enum represents the different types of operations that can be bundled
/// and executed through the Magic program.
pub enum MagicIntent<'args> {
    /// Standalone actions to execute on base layer without commit/undelegate semantics.
    StandaloneActions(NoVec<CallHandler<'args>, MAX_ACTIONS_NUM>),
    /// Commit accounts to base layer, optionally with post-commit actions.
    Commit(CommitIntent<'args>),
    /// Commit accounts and undelegate them, optionally with post-commit and post-undelegate actions.
    CommitAndUndelegate(CommitAndUndelegateIntent<'args>),
}

/// Builds a single `MagicBlockInstruction::ScheduleIntentBundle` instruction by aggregating
/// multiple independent intents (base actions, commits, commit+undelegate), normalizing them,
/// and producing a deduplicated account list plus the corresponding CPI `Instruction`.
pub struct MagicIntentBundleBuilder<'args> {
    payer: AccountView,
    magic_context: AccountView,
    magic_program: AccountView,
    intent_bundle: MagicIntentBundle<'args>,
}

impl<'args> MagicIntentBundleBuilder<'args> {
    pub fn new(payer: AccountView, magic_context: AccountView, magic_program: AccountView) -> Self {
        Self {
            payer,
            magic_context,
            magic_program,
            intent_bundle: MagicIntentBundle::default(),
        }
    }

    /// Starts building a Commit intent. Returns a [`CommitIntentBuilder`] that owns this parent.
    ///
    /// The returned builder lets you chain `.add_post_commit_actions()`, transition to other
    /// intents via `.commit_and_undelegate()`, or finalize via `.build_and_invoke()`.
    pub fn commit<'a>(self, accounts: &'a [AccountView]) -> CommitIntentBuilder<'a, 'args> {
        CommitIntentBuilder {
            parent: self,
            accounts,
            actions: NoVec::<CallHandler, MAX_ACTIONS_NUM>::new(),
        }
    }

    /// Starts building a CommitAndUndelegate intent. Returns a [`CommitAndUndelegateIntentBuilder`]
    /// that owns this parent.
    ///
    /// The returned builder lets you chain `.add_post_commit_actions()`,
    /// `.add_post_undelegate_actions()`, transition to other intents via `.commit()`,
    /// or finalize via `.build_and_invoke()`.
    pub fn commit_and_undelegate<'a>(
        self,
        accounts: &'a [AccountView],
    ) -> CommitAndUndelegateIntentBuilder<'a, 'args> {
        CommitAndUndelegateIntentBuilder {
            parent: self,
            accounts,
            post_commit_actions: NoVec::default(),
            post_undelegate_actions: NoVec::default(),
        }
    }

    /// Adds an intent to the bundle.
    ///
    /// If an intent of the same category already exists in the bundle:
    /// - base actions are appended
    /// - commit intents are merged (unique accounts/actions appended)
    /// - commit+undelegate intents are merged (unique accounts/actions appended)
    ///
    /// See `MagicIntentBundle::add_intent` for merge semantics.
    pub fn add_intent(mut self, intent: MagicIntent<'args>) -> Self {
        self.intent_bundle.add_intent(intent);
        self
    }

    /// Adds (or merges) a `Commit` intent into the bundle.
    pub fn add_commit(mut self, commit: CommitIntent<'args>) -> Self {
        self.intent_bundle.add_intent(MagicIntent::Commit(commit));
        self
    }

    /// Adds (or merges) a `CommitAndUndelegate` intent into the bundle.
    pub fn add_commit_and_undelegate(mut self, value: CommitAndUndelegateIntent<'args>) -> Self {
        self.intent_bundle
            .add_intent(MagicIntent::CommitAndUndelegate(value));
        self
    }

    /// Adds standalone base-layer actions to be executed without any commit/undelegate semantics.
    pub fn add_standalone_actions<'newargs>(
        self,
        actions: &[CallHandler<'newargs>],
    ) -> MagicIntentBundleBuilder<'newargs>
    where
        'args: 'newargs,
    {
        let mut this = MagicIntentBundleBuilder::<'newargs> {
            payer: self.payer,
            magic_program: self.magic_program,
            magic_context: self.magic_context,
            intent_bundle: self.intent_bundle,
        };

        let mut standalone_actions = NoVec::<CallHandler<'newargs>, MAX_ACTIONS_NUM>::new();
        standalone_actions.append_slice(actions);
        this.intent_bundle
            .add_intent(MagicIntent::StandaloneActions(standalone_actions));
        this
    }

    /// Normalizes the bundle, serializes it with bincode into `data_buf`, builds the
    /// CPI instruction, and invokes the magic program.
    ///
    /// `data_buf` must be large enough to hold the serialized `MagicIntentBundleArgs`.
    pub fn build_and_invoke(mut self, data_buf: &mut [u8]) -> ProgramResult {
        // 1. Normalize: dedup within intents, resolve cross-intent overlaps
        self.intent_bundle.normalize();

        // 2. Collect all unique accounts (payer + context first, then from intents)
        let mut all_accounts = NoVec::<AccountView, MAX_ACCOUNTS>::new();
        all_accounts.append([self.payer, self.magic_context]);
        self.intent_bundle
            .collect_unique_accounts(&mut all_accounts);

        // 3. Build the natural indices map: indices_map[i] = address of account at position i
        let mut indices_map = NoVec::<Address, MAX_ACCOUNTS>::new();
        for account in all_accounts.iter() {
            indices_map.push(account.address().clone());
        }

        // 4. Convert intents to serializable args
        let args = self.intent_bundle.into_args(indices_map.as_slice());

        // 5. Serialize with bincode (legacy config for bincode 1.x wire compat)
        let bytes_written =
            bincode::encode_into_slice(&args, data_buf, bincode::config::legacy())
                .map_err(|_| ProgramError::InvalidInstructionData)?;

        // 6. Build instruction account metas
        let mut instruction_accounts = NoVec::<InstructionAccount, MAX_ACCOUNTS>::new();
        for account in all_accounts.iter() {
            instruction_accounts.push(InstructionAccount::from(account));
        }

        // 7. Build instruction view
        let ix = InstructionView {
            program_id: self.magic_program.address(),
            data: &data_buf[..bytes_written],
            accounts: instruction_accounts.as_slice(),
        };

        // 8. Build account refs for invoke
        let mut account_refs = NoVec::<&AccountView, MAX_ACCOUNTS>::new();
        for account in all_accounts.iter() {
            account_refs.push(account);
        }

        // 9. Invoke CPI
        // NOTE: MAX_STATIC_CPI_ACCOUNTS = 64; if you need > 64 accounts,
        // enable `slice-cpi` feature and use `invoke_with_slice` instead.
        invoke_with_bounds::<64>(&ix, account_refs.as_slice())
    }
}

// ---------------------------------------------------------------------------
// Bundle of Intents
// ---------------------------------------------------------------------------

/// Bundle of Intents
///
/// Note: if `CommitIntent` & `CommitAndUndelegateIntent` has an account overlap
/// they will be undelegated
///
/// Intents assumed to be independent and self-sufficient,
/// hence order in which they were inserted doesn't matter
#[derive(Default)]
struct MagicIntentBundle<'args> {
    standalone_actions: NoVec<CallHandler<'args>, MAX_ACTIONS_NUM>,
    commit_intent: Option<CommitIntent<'args>>,
    commit_and_undelegate_intent: Option<CommitAndUndelegateIntent<'args>>,
}

impl<'args> MagicIntentBundle<'args> {
    /// Inserts an intent into the bundle, merging with any existing intent of the same category.
    fn add_intent(&mut self, intent: MagicIntent<'args>) {
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
    fn into_args(self, indices_map: &[Address]) -> MagicIntentBundleArgs<'args> {
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
    fn collect_unique_accounts(&self, unique_accounts: &mut NoVec<AccountView, MAX_ACCOUNTS>) {
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
    fn normalize(&mut self) {
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

/// Builder of Commit Intent.
///
/// Created via [`MagicIntentBundleBuilder::commit()`]. Owns the parent builder
/// and returns it (or a sibling sub-builder) on every transition/terminal call.
pub struct CommitIntentBuilder<'a, 'args> {
    parent: MagicIntentBundleBuilder<'args>,
    accounts: &'a [AccountView],
    actions: NoVec<CallHandler<'args>, MAX_ACTIONS_NUM>,
}

impl<'a, 'args> CommitIntentBuilder<'a, 'args> {
    /// Adds post-commit actions. Chainable.
    pub fn add_post_commit_actions<'new_args>(
        self,
        actions: &[CallHandler<'new_args>],
    ) -> CommitIntentBuilder<'a, 'new_args>
    where
        'args: 'new_args,
    {
        let mut this = CommitIntentBuilder::<'a, 'new_args> {
            parent: self.parent,
            accounts: self.accounts,
            actions: self.actions,
        };
        this.actions.append_slice(actions);
        this
    }

    /// Transition: finalizes this commit intent and starts a commit-and-undelegate intent.
    pub fn commit_and_undelegate<'cau>(
        self,
        accounts: &'cau [AccountView],
    ) -> CommitAndUndelegateIntentBuilder<'cau, 'args> {
        self.done().commit_and_undelegate(accounts)
    }

    /// Transition: finalizes this commit intent and adds standalone base-layer actions.
    pub fn add_standalone_actions<'newargs>(
        self,
        actions: &[CallHandler<'newargs>],
    ) -> MagicIntentBundleBuilder<'newargs>
    where
        'args: 'newargs,
    {
        self.done().add_standalone_actions(actions)
    }

    /// Terminal: finalizes this commit intent, builds the instruction and invokes it.
    pub fn build_and_invoke(self, data_buf: &mut [u8]) -> ProgramResult {
        self.done().build_and_invoke(data_buf)
    }

    /// Finalizes this commit intent and folds it into the parent bundle.
    fn done(self) -> MagicIntentBundleBuilder<'args> {
        let Self {
            mut parent,
            accounts: committed_accounts,
            actions,
        } = self;

        let mut accounts = NoVec::<AccountView, MAX_COMMITTED_ACCOUNTS_NUM>::new();
        accounts.append_slice(committed_accounts);
        let commit = CommitIntent { accounts, actions };

        parent.intent_bundle.add_intent(MagicIntent::Commit(commit));
        parent
    }
}

/// Builder of CommitAndUndelegate Intent.
///
/// Created via [`MagicIntentBundleBuilder::commit_and_undelegate()`] or
/// [`CommitIntentBuilder::commit_and_undelegate()`]. Owns the parent builder
/// and returns it (or a sibling sub-builder) on every transition/terminal call.
pub struct CommitAndUndelegateIntentBuilder<'a, 'args> {
    parent: MagicIntentBundleBuilder<'args>,
    accounts: &'a [AccountView],
    post_commit_actions: NoVec<CallHandler<'args>, MAX_ACTIONS_NUM>,
    post_undelegate_actions: NoVec<CallHandler<'args>, MAX_ACTIONS_NUM>,
}

impl<'a, 'args> CommitAndUndelegateIntentBuilder<'a, 'args> {
    // TODO: have slice & fixed-array version
    /// Adds post-commit actions. Chainable.
    pub fn add_post_commit_actions<'new_args>(
        self,
        actions: &[CallHandler<'new_args>],
    ) -> CommitAndUndelegateIntentBuilder<'a, 'new_args>
    where
        'args: 'new_args,
    {
        let mut this = CommitAndUndelegateIntentBuilder::<'a, 'new_args> {
            parent: self.parent,
            accounts: self.accounts,
            post_commit_actions: self.post_commit_actions,
            post_undelegate_actions: self.post_undelegate_actions,
        };
        this.post_commit_actions.append_slice(actions);
        this
    }

    /// Adds post-undelegate actions. Chainable.
    pub fn add_post_undelegate_actions<'new_args>(
        self,
        actions: &[CallHandler<'new_args>],
    ) -> CommitAndUndelegateIntentBuilder<'a, 'new_args>
    where
        'args: 'new_args,
    {
        let mut this = CommitAndUndelegateIntentBuilder::<'a, 'new_args> {
            parent: self.parent,
            accounts: self.accounts,
            post_commit_actions: self.post_commit_actions,
            post_undelegate_actions: self.post_undelegate_actions,
        };
        this.post_undelegate_actions.append_slice(actions);
        this
    }

    /// Transition: finalizes this commit-and-undelegate intent and starts a new commit intent.
    pub fn commit<'b>(self, accounts: &'b [AccountView]) -> CommitIntentBuilder<'b, 'args> {
        self.done().commit(accounts)
    }

    /// Transition: finalizes this commit-and-undelegate intent and adds standalone base-layer actions.
    pub fn add_standalone_actions<'newargs>(
        self,
        actions: &[CallHandler<'newargs>],
    ) -> MagicIntentBundleBuilder<'newargs>
    where
        'args: 'newargs,
    {
        self.done().add_standalone_actions(actions)
    }

    /// Terminal: finalizes this intent, builds the instruction and invokes it.
    pub fn build_and_invoke(self, data_buf: &mut [u8]) -> ProgramResult {
        self.done().build_and_invoke(data_buf)
    }

    /// Finalizes this commit-and-undelegate intent and folds it into the parent bundle.
    fn done(self) -> MagicIntentBundleBuilder<'args> {
        let Self {
            mut parent,
            accounts: committed_accounts,
            post_commit_actions,
            post_undelegate_actions,
        } = self;

        let mut accounts = NoVec::<_, MAX_COMMITTED_ACCOUNTS_NUM>::new();
        accounts.append_slice(committed_accounts);
        let cau = CommitAndUndelegateIntent {
            accounts,
            post_commit_actions,
            post_undelegate_actions,
        };
        parent
            .intent_bundle
            .add_intent(MagicIntent::CommitAndUndelegate(cau));
        parent
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

    fn collect_unique_accounts(&self, container: &mut NoVec<AccountView, MAX_ACCOUNTS>) {
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
    accounts: NoVec<AccountView, MAX_COMMITTED_ACCOUNTS_NUM>,
    actions: NoVec<CallHandler<'args>, MAX_ACTIONS_NUM>,
}

impl<'args> CommitIntent<'args> {
    fn committed_accounts(&self) -> &NoVec<AccountView, MAX_COMMITTED_ACCOUNTS_NUM> {
        &self.accounts
    }

    fn committed_accounts_mut(&mut self) -> &mut NoVec<AccountView, MAX_COMMITTED_ACCOUNTS_NUM> {
        &mut self.accounts
    }

    /// Deduplicates committed accounts by address. Accounts whose address is
    /// already in `seen` are removed. Newly seen addresses are added to `seen`.
    fn dedup(&mut self, seen: &mut NoVec<Address, MAX_ACCOUNTS>) {
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
            if !self.accounts.as_slice().iter().any(|a| a.address() == account.address()) {
                self.accounts.push(account);
            }
        }
        for action in other.actions {
            self.actions.push(action);
        }
    }

    fn collect_unique_accounts(&self, container: &mut NoVec<AccountView, MAX_ACCOUNTS>) {
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
        let mut indices = NoVec::<u8, MAX_COMMITTED_ACCOUNTS_NUM>::new();
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

// TODO: rename to CommitAndUndelegateIntent
pub struct CommitAndUndelegateIntent<'args> {
    accounts: NoVec<AccountView, MAX_COMMITTED_ACCOUNTS_NUM>,
    post_commit_actions: NoVec<CallHandler<'args>, MAX_ACTIONS_NUM>,
    post_undelegate_actions: NoVec<CallHandler<'args>, MAX_ACTIONS_NUM>,
}

impl<'args> CommitAndUndelegateIntent<'args> {
    /// Deduplicates committed accounts by address and returns the set of
    /// unique addresses (for cross-intent overlap detection).
    fn dedup(&mut self) -> NoVec<Address, MAX_ACCOUNTS> {
        let mut seen = NoVec::<Address, MAX_ACCOUNTS>::new();
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
            if !self.accounts.as_slice().iter().any(|a| a.address() == account.address()) {
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

    fn collect_unique_accounts(&self, container: &mut NoVec<AccountView, MAX_ACCOUNTS>) {
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
        let mut indices = NoVec::<u8, MAX_COMMITTED_ACCOUNTS_NUM>::new();
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

// ---------------------------------------------------------------------------
// Test-only: serialize builder output without CPI
// ---------------------------------------------------------------------------

#[cfg(test)]
impl<'args> MagicIntentBundleBuilder<'args> {
    /// Reproduces the logic of `build_and_invoke` but returns the serialized
    /// `MagicIntentBundleArgs` bytes instead of invoking CPI.
    fn build_serialized(mut self) -> alloc::vec::Vec<u8> {
        self.intent_bundle.normalize();

        let mut all_accounts = NoVec::<AccountView, MAX_ACCOUNTS>::new();
        all_accounts.append([self.payer, self.magic_context]);
        self.intent_bundle
            .collect_unique_accounts(&mut all_accounts);

        let mut indices_map = NoVec::<Address, MAX_ACCOUNTS>::new();
        for account in all_accounts.iter() {
            indices_map.push(account.address().clone());
        }

        let args = self.intent_bundle.into_args(indices_map.as_slice());
        let mut buf = alloc::vec![0u8; 4096];
        let len =
            bincode::encode_into_slice(&args, &mut buf, bincode::config::legacy()).unwrap();
        buf.truncate(len);
        buf
    }
}

#[cfg(test)]
impl<'a, 'args> CommitIntentBuilder<'a, 'args> {
    fn build_serialized(self) -> alloc::vec::Vec<u8> {
        self.done().build_serialized()
    }
}

#[cfg(test)]
impl<'a, 'args> CommitAndUndelegateIntentBuilder<'a, 'args> {
    fn build_serialized(self) -> alloc::vec::Vec<u8> {
        self.done().build_serialized()
    }
}

// ---------------------------------------------------------------------------
// Tests: builder compatibility between pinocchio and SDK
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    extern crate std;

    use std::cell::RefCell;
    use std::rc::Rc;
    use std::vec;
    use std::vec::Vec;

    use super::*;
    use crate::intent_bundle::args::ShortAccountMeta;

    // SDK builder
    use ephemeral_rollups_sdk::ephem::{
        CallHandler as SdkCallHandler, MagicIntentBundleBuilder as SdkBuilder,
    };
    use magicblock_magic_program_api::args::{
        ActionArgs as SdkActionArgs, ShortAccountMeta as SdkShortAccountMeta,
    };
    use magicblock_magic_program_api::Pubkey;
    use solana_program::account_info::AccountInfo;

    // -----------------------------------------------------------------
    // Mock helpers
    // -----------------------------------------------------------------

    /// Memory layout matching `RuntimeAccount` from `solana-account-view`.
    /// Used to back a pinocchio `AccountView` in tests.
    #[repr(C)]
    struct MockRuntimeAccount {
        borrow_state: u8,
        is_signer: u8,
        is_writable: u8,
        executable: u8,
        resize_delta: i32,
        address: [u8; 32],
        owner: [u8; 32],
        lamports: u64,
        data_len: u64,
    }

    impl MockRuntimeAccount {
        fn new(address: [u8; 32]) -> Self {
            Self {
                borrow_state: 0xFF, // NOT_BORROWED
                is_signer: 0,
                is_writable: 1,
                executable: 0,
                resize_delta: 0,
                address,
                owner: [0; 32],
                lamports: 1_000_000,
                data_len: 0,
            }
        }

        fn as_account_view(&mut self) -> AccountView {
            // SAFETY: MockRuntimeAccount has the same #[repr(C)] layout as
            // RuntimeAccount from solana-account-view. AccountView is a
            // #[repr(C)] wrapper around *mut RuntimeAccount.
            unsafe { core::mem::transmute(self as *mut Self) }
        }
    }

    /// Helper to hold owned data for an SDK `AccountInfo`.
    struct SdkTestAccount {
        key: Pubkey,
        lamports: u64,
        data: Vec<u8>,
        owner: Pubkey,
    }

    impl SdkTestAccount {
        fn new(address: [u8; 32]) -> Self {
            Self {
                key: Pubkey::new_from_array(address),
                lamports: 1_000_000,
                data: vec![],
                owner: Pubkey::new_from_array([0; 32]),
            }
        }

        fn as_account_info(&mut self) -> AccountInfo<'_> {
            AccountInfo {
                key: &self.key,
                is_signer: false,
                is_writable: true,
                lamports: Rc::new(RefCell::new(&mut self.lamports)),
                data: Rc::new(RefCell::new(&mut self.data)),
                owner: &self.owner,
                executable: false,
                rent_epoch: 0,
            }
        }

        fn as_signer_info(&mut self) -> AccountInfo<'_> {
            AccountInfo {
                key: &self.key,
                is_signer: true,
                is_writable: false,
                lamports: Rc::new(RefCell::new(&mut self.lamports)),
                data: Rc::new(RefCell::new(&mut self.data)),
                owner: &self.owner,
                executable: false,
                rent_epoch: 0,
            }
        }
    }

    /// Extract the `MagicIntentBundleArgs` bytes from SDK instruction data.
    ///
    /// The SDK wraps args in `MagicBlockInstruction::ScheduleIntentBundle(args)`,
    /// adding a 4-byte u32 LE enum discriminant prefix.
    fn extract_sdk_args(ix_data: &[u8]) -> &[u8] {
        &ix_data[4..]
    }

    // -----------------------------------------------------------------
    // Builder compatibility tests
    // -----------------------------------------------------------------

    /// Commit standalone (no actions).
    ///
    /// Both builders: `builder.commit(&[acc1, acc2]).build()`
    #[test]
    fn test_compat_commit_standalone() {
        let payer_addr = [0x01; 32];
        let ctx_addr = [0x02; 32];
        let acc1_addr = [0x03; 32];
        let acc2_addr = [0x04; 32];
        let prog_addr = [0xFF; 32];

        // --- Pinocchio builder ---
        let mut p_payer = MockRuntimeAccount::new(payer_addr);
        let mut p_ctx = MockRuntimeAccount::new(ctx_addr);
        let mut p_acc1 = MockRuntimeAccount::new(acc1_addr);
        let mut p_acc2 = MockRuntimeAccount::new(acc2_addr);
        let mut p_prog = MockRuntimeAccount::new(prog_addr);

        let commit_accs = [p_acc1.as_account_view(), p_acc2.as_account_view()];
        let pino_bytes = MagicIntentBundleBuilder::new(
            p_payer.as_account_view(),
            p_ctx.as_account_view(),
            p_prog.as_account_view(),
        )
        .commit(&commit_accs)
        .build_serialized();

        // --- SDK builder ---
        let mut s_payer = SdkTestAccount::new(payer_addr);
        let mut s_ctx = SdkTestAccount::new(ctx_addr);
        let mut s_acc1 = SdkTestAccount::new(acc1_addr);
        let mut s_acc2 = SdkTestAccount::new(acc2_addr);
        let mut s_prog = SdkTestAccount::new(prog_addr);

        let (accounts, ix) = SdkBuilder::new(
            s_payer.as_account_info(),
            s_ctx.as_account_info(),
            s_prog.as_account_info(),
        )
        .commit(&[s_acc1.as_account_info(), s_acc2.as_account_info()])
        .build();
        drop(accounts);

        assert_eq!(
            pino_bytes.as_slice(),
            extract_sdk_args(&ix.data),
            "commit standalone mismatch"
        );
    }

    /// Commit with a post-commit action (handler).
    #[test]
    fn test_compat_commit_with_handler() {
        let payer_addr = [0x01; 32];
        let ctx_addr = [0x02; 32];
        let acc1_addr = [0x03; 32];
        let escrow_addr = [0x04; 32];
        let dest_addr = [0xDD; 32];
        let prog_addr = [0xFF; 32];
        let action_data = [0xAA, 0xBB, 0xCC];

        // --- Pinocchio ---
        let mut p_payer = MockRuntimeAccount::new(payer_addr);
        let mut p_ctx = MockRuntimeAccount::new(ctx_addr);
        let mut p_acc1 = MockRuntimeAccount::new(acc1_addr);
        let mut p_escrow = MockRuntimeAccount::new(escrow_addr);
        let mut p_prog = MockRuntimeAccount::new(prog_addr);

        let escrow_view = p_escrow.as_account_view();
        let handler = CallHandler::new(
            Address::new_from_array(dest_addr),
            escrow_view,
            ActionArgs::new(&action_data),
            200_000,
        );
        let commit_accs = [p_acc1.as_account_view()];
        let pino_bytes = MagicIntentBundleBuilder::new(
            p_payer.as_account_view(),
            p_ctx.as_account_view(),
            p_prog.as_account_view(),
        )
        .commit(&commit_accs)
        .add_post_commit_actions(&[handler])
        .build_serialized();

        // --- SDK ---
        let mut s_payer = SdkTestAccount::new(payer_addr);
        let mut s_ctx = SdkTestAccount::new(ctx_addr);
        let mut s_acc1 = SdkTestAccount::new(acc1_addr);
        let mut s_escrow = SdkTestAccount::new(escrow_addr);
        let mut s_prog = SdkTestAccount::new(prog_addr);

        let sdk_handler = SdkCallHandler {
            args: SdkActionArgs::new(action_data.to_vec()),
            compute_units: 200_000,
            escrow_authority: s_escrow.as_signer_info(),
            destination_program: Pubkey::new_from_array(dest_addr),
            accounts: vec![],
        };
        let (accounts, ix) = SdkBuilder::new(
            s_payer.as_account_info(),
            s_ctx.as_account_info(),
            s_prog.as_account_info(),
        )
        .commit(&[s_acc1.as_account_info()])
        .add_post_commit_actions([sdk_handler])
        .build();
        drop(accounts);

        assert_eq!(
            pino_bytes.as_slice(),
            extract_sdk_args(&ix.data),
            "commit with handler mismatch"
        );
    }

    /// CommitAndUndelegate standalone (no actions).
    #[test]
    fn test_compat_commit_and_undelegate_standalone() {
        let payer_addr = [0x01; 32];
        let ctx_addr = [0x02; 32];
        let acc1_addr = [0x03; 32];
        let prog_addr = [0xFF; 32];

        // --- Pinocchio ---
        let mut p_payer = MockRuntimeAccount::new(payer_addr);
        let mut p_ctx = MockRuntimeAccount::new(ctx_addr);
        let mut p_acc1 = MockRuntimeAccount::new(acc1_addr);
        let mut p_prog = MockRuntimeAccount::new(prog_addr);

        let cau_accs = [p_acc1.as_account_view()];
        let pino_bytes = MagicIntentBundleBuilder::new(
            p_payer.as_account_view(),
            p_ctx.as_account_view(),
            p_prog.as_account_view(),
        )
        .commit_and_undelegate(&cau_accs)
        .build_serialized();

        // --- SDK ---
        let mut s_payer = SdkTestAccount::new(payer_addr);
        let mut s_ctx = SdkTestAccount::new(ctx_addr);
        let mut s_acc1 = SdkTestAccount::new(acc1_addr);
        let mut s_prog = SdkTestAccount::new(prog_addr);

        let (accounts, ix) = SdkBuilder::new(
            s_payer.as_account_info(),
            s_ctx.as_account_info(),
            s_prog.as_account_info(),
        )
        .commit_and_undelegate(&[s_acc1.as_account_info()])
        .build();
        drop(accounts);

        assert_eq!(
            pino_bytes.as_slice(),
            extract_sdk_args(&ix.data),
            "commit_and_undelegate standalone mismatch"
        );
    }

    /// CommitAndUndelegate with post-commit and post-undelegate actions.
    #[test]
    fn test_compat_commit_and_undelegate_with_actions() {
        let payer_addr = [0x01; 32];
        let ctx_addr = [0x02; 32];
        let acc1_addr = [0x03; 32];
        let escrow1_addr = [0x04; 32];
        let escrow2_addr = [0x05; 32];
        let dest1_addr = [0xAA; 32];
        let dest2_addr = [0xBB; 32];
        let prog_addr = [0xFF; 32];
        let commit_data = [1u8, 2, 3];
        let undelegate_data = [4u8, 5, 6];

        // --- Pinocchio ---
        let mut p_payer = MockRuntimeAccount::new(payer_addr);
        let mut p_ctx = MockRuntimeAccount::new(ctx_addr);
        let mut p_acc1 = MockRuntimeAccount::new(acc1_addr);
        let mut p_escrow1 = MockRuntimeAccount::new(escrow1_addr);
        let mut p_escrow2 = MockRuntimeAccount::new(escrow2_addr);
        let mut p_prog = MockRuntimeAccount::new(prog_addr);

        let post_commit = CallHandler::new(
            Address::new_from_array(dest1_addr),
            p_escrow1.as_account_view(),
            ActionArgs::new(&commit_data),
            100_000,
        );
        let post_undelegate = CallHandler::new(
            Address::new_from_array(dest2_addr),
            p_escrow2.as_account_view(),
            ActionArgs::new(&undelegate_data),
            50_000,
        );
        let cau_accs = [p_acc1.as_account_view()];
        let pino_bytes = MagicIntentBundleBuilder::new(
            p_payer.as_account_view(),
            p_ctx.as_account_view(),
            p_prog.as_account_view(),
        )
        .commit_and_undelegate(&cau_accs)
        .add_post_commit_actions(&[post_commit])
        .add_post_undelegate_actions(&[post_undelegate])
        .build_serialized();

        // --- SDK ---
        let mut s_payer = SdkTestAccount::new(payer_addr);
        let mut s_ctx = SdkTestAccount::new(ctx_addr);
        let mut s_acc1 = SdkTestAccount::new(acc1_addr);
        let mut s_escrow1 = SdkTestAccount::new(escrow1_addr);
        let mut s_escrow2 = SdkTestAccount::new(escrow2_addr);
        let mut s_prog = SdkTestAccount::new(prog_addr);

        let sdk_post_commit = SdkCallHandler {
            args: SdkActionArgs::new(commit_data.to_vec()),
            compute_units: 100_000,
            escrow_authority: s_escrow1.as_signer_info(),
            destination_program: Pubkey::new_from_array(dest1_addr),
            accounts: vec![],
        };
        let sdk_post_undelegate = SdkCallHandler {
            args: SdkActionArgs::new(undelegate_data.to_vec()),
            compute_units: 50_000,
            escrow_authority: s_escrow2.as_signer_info(),
            destination_program: Pubkey::new_from_array(dest2_addr),
            accounts: vec![],
        };
        let (accounts, ix) = SdkBuilder::new(
            s_payer.as_account_info(),
            s_ctx.as_account_info(),
            s_prog.as_account_info(),
        )
        .commit_and_undelegate(&[s_acc1.as_account_info()])
        .add_post_commit_actions([sdk_post_commit])
        .add_post_undelegate_actions([sdk_post_undelegate])
        .build();
        drop(accounts);

        assert_eq!(
            pino_bytes.as_slice(),
            extract_sdk_args(&ix.data),
            "commit_and_undelegate with actions mismatch"
        );
    }

    /// Standalone actions only (no commit / undelegate).
    #[test]
    fn test_compat_standalone_actions() {
        let payer_addr = [0x01; 32];
        let ctx_addr = [0x02; 32];
        let escrow_addr = [0x04; 32];
        let dest_addr = [0xA1; 32];
        let extra_addr = [0xB1; 32];
        let prog_addr = [0xFF; 32];
        let data = [0x10u8, 0x20];

        // --- Pinocchio ---
        let mut p_payer = MockRuntimeAccount::new(payer_addr);
        let mut p_ctx = MockRuntimeAccount::new(ctx_addr);
        let mut p_escrow = MockRuntimeAccount::new(escrow_addr);
        let mut p_prog = MockRuntimeAccount::new(prog_addr);

        let mut handler = CallHandler::new(
            Address::new_from_array(dest_addr),
            p_escrow.as_account_view(),
            ActionArgs::new(&data),
            100_000,
        );
        handler.add_accounts_slice(&[ShortAccountMeta {
            pubkey: Address::new_from_array(extra_addr),
            is_writable: true,
        }]);
        let pino_bytes = MagicIntentBundleBuilder::new(
            p_payer.as_account_view(),
            p_ctx.as_account_view(),
            p_prog.as_account_view(),
        )
        .add_standalone_actions(&[handler])
        .build_serialized();

        // --- SDK ---
        let mut s_payer = SdkTestAccount::new(payer_addr);
        let mut s_ctx = SdkTestAccount::new(ctx_addr);
        let mut s_escrow = SdkTestAccount::new(escrow_addr);
        let mut s_prog = SdkTestAccount::new(prog_addr);

        let sdk_handler = SdkCallHandler {
            args: SdkActionArgs::new(data.to_vec()),
            compute_units: 100_000,
            escrow_authority: s_escrow.as_signer_info(),
            destination_program: Pubkey::new_from_array(dest_addr),
            accounts: vec![SdkShortAccountMeta {
                pubkey: Pubkey::new_from_array(extra_addr),
                is_writable: true,
            }],
        };
        let (accounts, ix) = SdkBuilder::new(
            s_payer.as_account_info(),
            s_ctx.as_account_info(),
            s_prog.as_account_info(),
        )
        .add_standalone_actions([sdk_handler])
        .build();
        drop(accounts);

        assert_eq!(
            pino_bytes.as_slice(),
            extract_sdk_args(&ix.data),
            "standalone actions mismatch"
        );
    }

    /// Chained: commit then commit_and_undelegate.
    #[test]
    fn test_compat_commit_then_commit_and_undelegate() {
        let payer_addr = [0x01; 32];
        let ctx_addr = [0x02; 32];
        let acc1_addr = [0x03; 32];
        let acc2_addr = [0x04; 32];
        let prog_addr = [0xFF; 32];

        // --- Pinocchio ---
        let mut p_payer = MockRuntimeAccount::new(payer_addr);
        let mut p_ctx = MockRuntimeAccount::new(ctx_addr);
        let mut p_acc1 = MockRuntimeAccount::new(acc1_addr);
        let mut p_acc2 = MockRuntimeAccount::new(acc2_addr);
        let mut p_prog = MockRuntimeAccount::new(prog_addr);

        let commit_accs = [p_acc1.as_account_view()];
        let cau_accs = [p_acc2.as_account_view()];
        let pino_bytes = MagicIntentBundleBuilder::new(
            p_payer.as_account_view(),
            p_ctx.as_account_view(),
            p_prog.as_account_view(),
        )
        .commit(&commit_accs)
        .commit_and_undelegate(&cau_accs)
        .build_serialized();

        // --- SDK ---
        let mut s_payer = SdkTestAccount::new(payer_addr);
        let mut s_ctx = SdkTestAccount::new(ctx_addr);
        let mut s_acc1 = SdkTestAccount::new(acc1_addr);
        let mut s_acc2 = SdkTestAccount::new(acc2_addr);
        let mut s_prog = SdkTestAccount::new(prog_addr);

        let (accounts, ix) = SdkBuilder::new(
            s_payer.as_account_info(),
            s_ctx.as_account_info(),
            s_prog.as_account_info(),
        )
        .commit(&[s_acc1.as_account_info()])
        .commit_and_undelegate(&[s_acc2.as_account_info()])
        .build();
        drop(accounts);

        assert_eq!(
            pino_bytes.as_slice(),
            extract_sdk_args(&ix.data),
            "commit then commit_and_undelegate mismatch"
        );
    }

    /// All intent types combined: commit + commit_and_undelegate + standalone actions.
    #[test]
    fn test_compat_all_intents_combined() {
        let payer_addr = [0x01; 32];
        let ctx_addr = [0x02; 32];
        let commit_acc_addr = [0x03; 32];
        let cau_acc_addr = [0x04; 32];
        let escrow_addr = [0x05; 32];
        let dest_addr = [0xE1; 32];
        let prog_addr = [0xFF; 32];
        let standalone_data = [0xE0u8];

        // --- Pinocchio ---
        let mut p_payer = MockRuntimeAccount::new(payer_addr);
        let mut p_ctx = MockRuntimeAccount::new(ctx_addr);
        let mut p_commit = MockRuntimeAccount::new(commit_acc_addr);
        let mut p_cau = MockRuntimeAccount::new(cau_acc_addr);
        let mut p_escrow = MockRuntimeAccount::new(escrow_addr);
        let mut p_prog = MockRuntimeAccount::new(prog_addr);

        let handler = CallHandler::new(
            Address::new_from_array(dest_addr),
            p_escrow.as_account_view(),
            ActionArgs::new(&standalone_data),
            150_000,
        );
        let commit_accs = [p_commit.as_account_view()];
        let cau_accs = [p_cau.as_account_view()];
        let pino_bytes = MagicIntentBundleBuilder::new(
            p_payer.as_account_view(),
            p_ctx.as_account_view(),
            p_prog.as_account_view(),
        )
        .commit(&commit_accs)
        .commit_and_undelegate(&cau_accs)
        .add_standalone_actions(&[handler])
        .build_serialized();

        // --- SDK ---
        let mut s_payer = SdkTestAccount::new(payer_addr);
        let mut s_ctx = SdkTestAccount::new(ctx_addr);
        let mut s_commit = SdkTestAccount::new(commit_acc_addr);
        let mut s_cau = SdkTestAccount::new(cau_acc_addr);
        let mut s_escrow = SdkTestAccount::new(escrow_addr);
        let mut s_prog = SdkTestAccount::new(prog_addr);

        let sdk_handler = SdkCallHandler {
            args: SdkActionArgs::new(standalone_data.to_vec()),
            compute_units: 150_000,
            escrow_authority: s_escrow.as_signer_info(),
            destination_program: Pubkey::new_from_array(dest_addr),
            accounts: vec![],
        };
        let (accounts, ix) = SdkBuilder::new(
            s_payer.as_account_info(),
            s_ctx.as_account_info(),
            s_prog.as_account_info(),
        )
        .commit(&[s_commit.as_account_info()])
        .commit_and_undelegate(&[s_cau.as_account_info()])
        .add_standalone_actions([sdk_handler])
        .build();
        drop(accounts);

        assert_eq!(
            pino_bytes.as_slice(),
            extract_sdk_args(&ix.data),
            "all intents combined mismatch"
        );
    }

    /// Full chain with actions on all intents.
    #[test]
    fn test_compat_full_chain_with_actions() {
        let payer_addr = [0x01; 32];
        let ctx_addr = [0x02; 32];
        let commit_acc_addr = [0x03; 32];
        let cau_acc_addr = [0x04; 32];
        let escrow1_addr = [0x05; 32];
        let escrow2_addr = [0x06; 32];
        let dest1_addr = [0xC1; 32];
        let dest2_addr = [0xD1; 32];
        let prog_addr = [0xFF; 32];
        let commit_data = [0xC0u8];
        let undelegate_data = [0xD0u8];

        // --- Pinocchio ---
        let mut p_payer = MockRuntimeAccount::new(payer_addr);
        let mut p_ctx = MockRuntimeAccount::new(ctx_addr);
        let mut p_commit = MockRuntimeAccount::new(commit_acc_addr);
        let mut p_cau = MockRuntimeAccount::new(cau_acc_addr);
        let mut p_escrow1 = MockRuntimeAccount::new(escrow1_addr);
        let mut p_escrow2 = MockRuntimeAccount::new(escrow2_addr);
        let mut p_prog = MockRuntimeAccount::new(prog_addr);

        let commit_handler = CallHandler::new(
            Address::new_from_array(dest1_addr),
            p_escrow1.as_account_view(),
            ActionArgs::new(&commit_data),
            100_000,
        );
        let undelegate_handler = CallHandler::new(
            Address::new_from_array(dest2_addr),
            p_escrow2.as_account_view(),
            ActionArgs::new(&undelegate_data),
            50_000,
        );
        let commit_accs = [p_commit.as_account_view()];
        let cau_accs = [p_cau.as_account_view()];
        let pino_bytes = MagicIntentBundleBuilder::new(
            p_payer.as_account_view(),
            p_ctx.as_account_view(),
            p_prog.as_account_view(),
        )
        .commit(&commit_accs)
        .add_post_commit_actions(&[commit_handler])
        .commit_and_undelegate(&cau_accs)
        .add_post_undelegate_actions(&[undelegate_handler])
        .build_serialized();

        // --- SDK ---
        let mut s_payer = SdkTestAccount::new(payer_addr);
        let mut s_ctx = SdkTestAccount::new(ctx_addr);
        let mut s_commit = SdkTestAccount::new(commit_acc_addr);
        let mut s_cau = SdkTestAccount::new(cau_acc_addr);
        let mut s_escrow1 = SdkTestAccount::new(escrow1_addr);
        let mut s_escrow2 = SdkTestAccount::new(escrow2_addr);
        let mut s_prog = SdkTestAccount::new(prog_addr);

        let sdk_commit_handler = SdkCallHandler {
            args: SdkActionArgs::new(commit_data.to_vec()),
            compute_units: 100_000,
            escrow_authority: s_escrow1.as_signer_info(),
            destination_program: Pubkey::new_from_array(dest1_addr),
            accounts: vec![],
        };
        let sdk_undelegate_handler = SdkCallHandler {
            args: SdkActionArgs::new(undelegate_data.to_vec()),
            compute_units: 50_000,
            escrow_authority: s_escrow2.as_signer_info(),
            destination_program: Pubkey::new_from_array(dest2_addr),
            accounts: vec![],
        };
        let (accounts, ix) = SdkBuilder::new(
            s_payer.as_account_info(),
            s_ctx.as_account_info(),
            s_prog.as_account_info(),
        )
        .commit(&[s_commit.as_account_info()])
        .add_post_commit_actions([sdk_commit_handler])
        .commit_and_undelegate(&[s_cau.as_account_info()])
        .add_post_undelegate_actions([sdk_undelegate_handler])
        .build();
        drop(accounts);

        assert_eq!(
            pino_bytes.as_slice(),
            extract_sdk_args(&ix.data),
            "full chain with actions mismatch"
        );
    }
}
