use crate::intent_bundle::args::{
    ActionArgs, BaseActionArgs, CommitAndUndelegateArgs, CommitType, CommitTypeArgs,
    MagicIntentBundleArgs, ShortAccountMeta, UndelegateType,
};
use crate::intent_bundle::no_vec::NoVec;
use alloc::vec;
use alloc::vec::Vec;
use pinocchio::error::ProgramError;
use pinocchio::instruction::InstructionView;
use pinocchio::{AccountView, ProgramResult};
use solana_address::Address;

mod args;
mod no_vec;
mod types;

const MAX_ACTIONS_NUM: usize = 10u8 as usize;
const MAX_COMMITTED_ACCOUNTS_NUM: usize = 64u8 as usize;
const MAX_ACCOUNTS: usize = u8::MAX as usize;
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
    CommitAndUndelegate(CommitAndUndelegate<'args>),
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
    /// The returned builder lets you chain `.t_commit_actions()`, transition to other
    /// intents via `.commit_and_undelegate()`, or finalize via `.build()` / `.build_and_invoke()`.
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
    /// or finalize via `.build()` / `.build_and_invoke()`.
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
    /// - commit intents are merged (accounts/actions appended; variant upgraded to handler if needed)
    /// - commit+undelegate intents are merged (accounts/actions appended)
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
    pub fn add_commit_and_undelegate(mut self, value: CommitAndUndelegate<'args>) -> Self {
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

    /// Builds the deduplicated account list and the CPI `Instruction` that schedules this bundle.
    ///
    /// # Returns
    /// - `Vec<AccountInfo>`: the full, deduplicated account list to pass to CPI (payer/context first).
    /// - `Instruction`: the instruction to invoke against the magic program.
    pub fn build(mut self) -> (NoVec<AccountView, MAX_ACCOUNTS>, InstructionView) {
        // Dedup Intent Bundle
        self.intent_bundle.normalize();

        // Collect all accounts used by the bundle, then dedup them + create index map.
        let mut all_accounts = NoVec::<AccountView, MAX_ACTIONS_NUM>::new();
        all_accounts.append([self.payer, self.magic_context]);
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
        invo(&ix, &accounts)
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
struct MagicIntentBundle<'args> {
    standalone_actions: NoVec<CallHandler<'args>, MAX_ACTIONS_NUM>,
    commit_intent: Option<CommitIntent<'args>>,
    commit_and_undelegate_intent: Option<CommitAndUndelegate<'args>>,
}

#[test]
pub fn test() {
    let bundle = MagicIntentBundle::default();
}

impl<'info> MagicIntentBundle<'info> {
    /// Inserts an intent into the bundle, merging with any existing intent of the same category.
    fn add_intent(&mut self, intent: MagicIntent<'info>) {
        match intent {
            MagicIntent::StandaloneActions(value) => self.standalone_actions.extend(value),
            MagicIntent::Commit(value) => {
                if let Some(ref mut commit_accounts) = self.commit_intent {
                    commit_accounts.merge(value);
                } else {
                    self.commit_intent = Some(value);
                }
            }
            MagicIntent::CommitAndUndelegate(value) => {
                if let Some(ref mut commit_and_undelegate) = self.commit_and_undelegate_intent {
                    commit_and_undelegate.merge(value);
                } else {
                    self.commit_and_undelegate_intent = Some(value);
                }
            }
        }
    }

    /// Consumes the bundle and encodes it into `MagicIntentBundleArgs` using a `Pubkey -> u8` indices map.
    fn into_args(self, indices_map: &[Address]) -> MagicIntentBundleArgs {
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
        mut self,
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

    /// Terminal: finalizes this commit intent and builds the full instruction.
    pub fn build(self) -> (Vec<AccountInfo<'info>>, Instruction) {
        self.done().build()
    }

    /// Terminal: finalizes this commit intent, builds the instruction and invokes it.
    pub fn build_and_invoke(self) -> ProgramResult {
        self.done().build_and_invoke()
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
pub struct CommitAndUndelegateIntentBuilder<'a, 'info> {
    parent: MagicIntentBundleBuilder<'info>,
    accounts: &'a [AccountView],
    post_commit_actions: NoVec<CallHandler<'info>, MAX_ACTIONS_NUM>,
    post_undelegate_actions: NoVec<CallHandler<'info>, MAX_ACTIONS_NUM>,
}

impl<'a, 'args> CommitAndUndelegateIntentBuilder<'a, 'args> {
    // TODO: have slice & fixed-array version
    /// Adds post-commit actions. Chainable.
    pub fn add_post_commit_actions<'new_args>(
        mut self,
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
        mut self,
        actions: &[CallHandler<'args>],
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

    /// Terminal: finalizes this intent and builds the full instruction.
    pub fn build(self) -> (Vec<AccountInfo<'info>>, Instruction) {
        self.done().build()
    }

    /// Terminal: finalizes this intent, builds the instruction and invokes it.
    pub fn build_and_invoke(self) -> ProgramResult {
        self.done().build_and_invoke()
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
        let cau = CommitAndUndelegate {
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

    fn add_accounts_slice(&mut self, accounts: &[ShortAccountMeta]) {
        self.accounts.append_slice(accounts);
    }

    fn add_accounts<const N: usize>(&mut self, accounts: [ShortAccountMeta; N]) {
        self.accounts.append(accounts);
    }

    pub(crate) fn into_args(self, indices_map: &[Address]) -> BaseActionArgs {
        let escrow_authority_index =
            get_index(indices_map, self.escrow_authority.address()).expect(EXPECTED_KEY_MSG);
        BaseActionArgs {
            args: self.args,
            compute_units: self.compute_units,
            destination_program: self.destination_program.to_bytes().into(),
            escrow_authority: *escrow_authority_index,
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

    fn dedup(&mut self, seen: &mut NoVec<Address, MAX_ACCOUNTS>) {
        let committed_accounts = self.committed_accounts_mut();
        committed_accounts.retain(|el| {
            if seen.contains(el.key()) {
                false
            } else {
                seen.push(*el.key());
                true
            }
        });
    }

    fn collect_accounts(&self, container: &mut NoVec<AccountView, MAX_ACCOUNTS>) {
        container.append_slice(self.accounts.as_slice());
    }

    fn into_args(self, indices_map: &[Address]) -> Result<CommitTypeArgs, ProgramError> {
        let mut indices = NoVec::<_, MAX_ACCOUNTS>::new();
        for account in self.accounts {
            let idx = get_index(indices_map, account.address())
                .ok_or(ProgramError::InvalidAccountData)?;
            indices.push(idx);
        }

        let res = if self.actions.is_empty() {
            CommitTypeArgs::Standalone(indices)
        } else {
            let mut base_actions = NoVec::<_, MAX_ACTIONS_NUM>::new();
            for handler in self.actions {
                base_actions.push(handler.into_args(indices_map)?);
            }
            CommitTypeArgs::WithBaseActions {
                committed_accounts: indices,
                base_actions,
            }
        };

        Ok(res)
    }

    fn merge(&mut self, other: Self) {
        let take = |value: &mut Self| -> (Vec<&'a AccountInfo>, Vec<BaseAction<'a>>) {
            match value {
                CommitType::Standalone(accounts) => (core::mem::take(accounts), vec![]),
                CommitType::WithHandler {
                    committed_accounts,
                    call_handlers,
                } => (
                    core::mem::take(committed_accounts),
                    core::mem::take(call_handlers),
                ),
            }
        };

        let (mut accounts, mut actions) = take(self);
        let (other_accounts, other_actions) = {
            let mut other = other;
            take(&mut other)
        };
        accounts.extend(other_accounts);
        actions.extend(other_actions);

        if actions.is_empty() {
            *self = CommitType::Standalone(accounts);
        } else {
            *self = CommitType::WithHandler {
                committed_accounts: accounts,
                call_handlers: actions,
            };
        }
    }
}

// TODO: rename to CommitAndUndelegateIntent
pub struct CommitAndUndelegate<'args> {
    accounts: NoVec<AccountView, MAX_COMMITTED_ACCOUNTS_NUM>,
    post_commit_actions: NoVec<CallHandler<'args>, MAX_ACTIONS_NUM>,
    post_undelegate_actions: NoVec<CallHandler<'args>, MAX_ACTIONS_NUM>,
}

impl<'a> CommitAndUndelegate<'a> {
    fn collect_accounts(&self, container: &mut Vec<AccountView>) {
        self.commit_type.collect_accounts(container);
        self.undelegate_type.collect_accounts(container);
    }

    fn into_args(self, pubkeys: &[Address]) -> Result<CommitAndUndelegateArgs, ProgramError> {
        let commit_type = self.commit_type.into_args(pubkeys)?;
        let undelegate_type = self.undelegate_type.into_args(pubkeys)?;
        Ok(CommitAndUndelegateArgs {
            commit_type,
            undelegate_type,
        })
    }

    fn dedup(&mut self) -> Vec<Address> {
        self.commit_type.dedup()
    }

    fn merge(&mut self, other: Self) {
        self.commit_type.merge(other.commit_type);

        let this = core::mem::replace(&mut self.undelegate_type, UndelegateType::Standalone);
        self.undelegate_type = match (this, other.undelegate_type) {
            (UndelegateType::Standalone, UndelegateType::Standalone) => UndelegateType::Standalone,
            (UndelegateType::Standalone, UndelegateType::WithHandler(v))
            | (UndelegateType::WithHandler(v), UndelegateType::Standalone) => {
                UndelegateType::WithHandler(v)
            }
            (UndelegateType::WithHandler(mut a), UndelegateType::WithHandler(b)) => {
                a.extend(b);
                UndelegateType::WithHandler(a)
            }
        };
    }
}

/// Gets the index of a pubkey in the deduplicated pubkey list.
/// Returns None if the pubkey is not found.
fn get_index(pubkeys: &[Address], needle: &Address) -> Option<u8> {
    pubkeys.iter().position(|k| k == needle).map(|i| i as u8)
}
