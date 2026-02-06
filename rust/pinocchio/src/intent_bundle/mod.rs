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
