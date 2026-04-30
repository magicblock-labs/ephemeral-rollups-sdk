use crate::intent_bundle::args::{
    ActionArgs, AddActionCallbackArgs, BaseActionArgs, CommitAndUndelegateArgs, CommitTypeArgs,
    MagicIntentBundleArgs, ShortAccountMeta, UndelegateTypeArgs,
};
use crate::intent_bundle::no_vec::NoVec;
use pinocchio::cpi::MAX_STATIC_CPI_ACCOUNTS;
use pinocchio::error::ProgramError;
use pinocchio::{AccountView, ProgramResult};
use solana_address::Address;

use super::MAX_ACTIONS_NUM;

/// Intent to be scheduled for execution on the base layer.
///
/// This enum represents the different types of operations that can be bundled
/// and executed through the Magic program.
#[allow(clippy::large_enum_variant)]
pub enum MagicIntent<'acc, 'args> {
    /// Standalone actions to execute on base layer without commit/undelegate semantics.
    StandaloneActions(&'args [CallHandler<'args>]),
    /// Commit accounts to base layer, optionally with post-commit actions.
    Commit(CommitIntent<'acc, 'args>),
    /// Commit accounts and undelegate them, optionally with post-commit and post-undelegate actions.
    CommitAndUndelegate(CommitAndUndelegateIntent<'acc, 'args>),
}

/// Bundle of Intents
///
/// Note: if `CommitIntent` & `CommitAndUndelegateIntent` has an account overlap
/// they will be undelegated
///
/// Intents assumed to be independent and self-sufficient,
/// hence order in which they were inserted doesn't matter
#[derive(Default)]
pub(in crate::intent_bundle) struct MagicIntentBundle<'acc, 'args> {
    pub(in crate::intent_bundle) standalone_actions: &'args [CallHandler<'args>],
    pub(in crate::intent_bundle) commit_intent: Option<CommitIntent<'acc, 'args>>,
    pub(in crate::intent_bundle) commit_and_undelegate_intent:
        Option<CommitAndUndelegateIntent<'acc, 'args>>,
    pub(in crate::intent_bundle) commit_finalize_compressed_intent:
        Option<CommitIntent<'acc, 'args>>,
    pub(in crate::intent_bundle) commit_finalize_compressed_and_undelegate_intent:
        Option<CommitAndUndelegateIntent<'acc, 'args>>,
}

impl<'args> MagicIntentBundle<'_, 'args> {
    /// Consumes the bundle and encodes it into `MagicIntentBundleArgs` using an indices map.
    ///
    /// `indices_map` is a natural map: `indices_map[i]` is the address at index `i`.
    #[allow(dead_code)]
    pub(super) fn into_args(
        self,
        indices_map: &[&Address],
    ) -> Result<MagicIntentBundleArgs<'args>, ProgramError> {
        let commit = self
            .commit_intent
            .map(|c| c.into_args(indices_map))
            .transpose()?;
        let commit_and_undelegate = self
            .commit_and_undelegate_intent
            .map(|c| c.into_args(indices_map))
            .transpose()?;
        let commit_finalize_compressed = self
            .commit_finalize_compressed_intent
            .map(|c| c.into_args(indices_map))
            .transpose()?;
        let commit_finalize_compressed_and_undelegate = self
            .commit_finalize_compressed_and_undelegate_intent
            .map(|c| c.into_args(indices_map))
            .transpose()?;
        let mut standalone_actions = NoVec::<BaseActionArgs<'args>, MAX_ACTIONS_NUM>::new();
        for ch in self.standalone_actions {
            standalone_actions.try_push(ch.args(indices_map)?)?;
        }

        Ok(MagicIntentBundleArgs {
            commit,
            commit_and_undelegate,
            commit_finalize: None,
            commit_finalize_and_undelegate: None,
            commit_finalize_compressed,
            commit_finalize_compressed_and_undelegate,
            standalone_actions,
        })
    }

    /// Collects all accounts referenced by intents in this bundle.
    #[inline(never)]
    pub(super) fn collect_unique_accounts(
        &self,
        unique_accounts: &mut NoVec<AccountView, MAX_STATIC_CPI_ACCOUNTS>,
    ) -> ProgramResult {
        for el in self.standalone_actions {
            el.collect_unique_accounts(unique_accounts)?;
        }
        if let Some(commit) = &self.commit_intent {
            commit.collect_unique_accounts(unique_accounts)?;
        }
        if let Some(cau) = &self.commit_and_undelegate_intent {
            cau.collect_unique_accounts(unique_accounts)?;
        }
        Ok(())
    }

    /// Validates the bundle:
    /// - Each present intent must have at least one committed account.
    /// - No duplicate accounts within an intent.
    /// - No account overlap between `Commit` and `CommitAndUndelegate`.
    #[inline(never)]
    pub(super) fn validate(&self) -> ProgramResult {
        let mut seen = NoVec::<Address, MAX_STATIC_CPI_ACCOUNTS>::new();
        if let Some(commit) = &self.commit_intent {
            commit.validate()?;
            commit.try_collect_unique_addresses(&mut seen)?;
        }
        if let Some(cau) = &self.commit_and_undelegate_intent {
            cau.validate()?;
            cau.try_collect_unique_addresses(&mut seen)?;
        }
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// Intent Types
// ---------------------------------------------------------------------------

#[derive(Copy, Clone)]
pub struct CommitIntent<'acc, 'args> {
    pub(super) accounts: &'acc [AccountView],
    pub(super) actions: &'args [CallHandler<'args>],
}

impl<'args> CommitIntent<'_, 'args> {
    /// Validates that this commit intent has at least one account to commit.
    fn validate(&self) -> ProgramResult {
        if self.accounts.is_empty() {
            Err(ProgramError::InvalidInstructionData)
        } else {
            Ok(())
        }
    }

    /// Validates that none of this intent's accounts are already in `seen`,
    /// then appends each address to `seen`.
    /// Returns `Err(InvalidArgument)` on duplicates within this intent or
    /// overlap with addresses already in `seen` (e.g. from another intent).
    #[allow(clippy::clone_on_copy)]
    fn try_collect_unique_addresses(
        &self,
        seen: &mut NoVec<Address, MAX_STATIC_CPI_ACCOUNTS>,
    ) -> ProgramResult {
        for account in self.accounts {
            let addr = account.address();
            if seen.contains(addr) {
                return Err(ProgramError::InvalidArgument);
            }
            seen.try_push(addr.clone())?;
        }
        Ok(())
    }

    #[inline(never)]
    fn collect_unique_accounts(
        &self,
        container: &mut NoVec<AccountView, MAX_STATIC_CPI_ACCOUNTS>,
    ) -> ProgramResult {
        for el in self.accounts.iter() {
            if !container.contains(el) {
                container.try_push(el.clone())?;
            }
        }
        for el in self.actions.iter() {
            el.collect_unique_accounts(container)?;
        }
        Ok(())
    }

    pub(in crate::intent_bundle) fn into_args(
        self,
        indices_map: &[&Address],
    ) -> Result<CommitTypeArgs<'args>, ProgramError> {
        let mut indices = NoVec::<u8, MAX_STATIC_CPI_ACCOUNTS>::new();
        for account in self.accounts {
            let idx =
                get_index(indices_map, account.address()).ok_or(ProgramError::InvalidArgument)?;
            indices.try_push(idx)?;
        }

        if self.actions.is_empty() {
            Ok(CommitTypeArgs::Standalone(indices))
        } else {
            let mut base_actions = NoVec::<_, MAX_ACTIONS_NUM>::new();
            for handler in self.actions {
                base_actions.try_push(handler.args(indices_map)?)?;
            }
            Ok(CommitTypeArgs::WithBaseActions {
                committed_accounts: indices,
                base_actions,
            })
        }
    }

    pub(crate) fn get_actions_len(&self) -> usize {
        self.actions.len()
    }

    pub(crate) fn get_action_callback(&self, ind: usize) -> Option<&ActionCallback<'args>> {
        self.actions.get(ind).and_then(|el| el.callback.as_ref())
    }
}

#[derive(Copy, Clone)]
pub struct CommitAndUndelegateIntent<'acc, 'args> {
    pub(super) accounts: &'acc [AccountView],
    pub(super) post_commit_actions: &'args [CallHandler<'args>],
    pub(super) post_undelegate_actions: &'args [CallHandler<'args>],
}

impl<'args> CommitAndUndelegateIntent<'_, 'args> {
    /// Validates that this commit-and-undelegate intent has at least one account.
    fn validate(&self) -> ProgramResult {
        if self.accounts.is_empty() {
            Err(ProgramError::InvalidInstructionData)
        } else {
            Ok(())
        }
    }

    /// Validates that none of this intent's accounts are already in `seen`,
    /// then appends each address to `seen`.
    /// Returns `Err(InvalidArgument)` on duplicates within this intent or
    /// overlap with addresses already in `seen` (e.g. from another intent).
    #[allow(clippy::clone_on_copy)]
    fn try_collect_unique_addresses(
        &self,
        seen: &mut NoVec<Address, MAX_STATIC_CPI_ACCOUNTS>,
    ) -> ProgramResult {
        for account in self.accounts {
            let addr = account.address();
            if seen.contains(addr) {
                return Err(ProgramError::InvalidArgument);
            }
            seen.try_push(addr.clone())?;
        }
        Ok(())
    }

    #[inline(never)]
    fn collect_unique_accounts(
        &self,
        container: &mut NoVec<AccountView, MAX_STATIC_CPI_ACCOUNTS>,
    ) -> ProgramResult {
        for el in self.accounts.iter() {
            if !container.contains(el) {
                container.try_push(el.clone())?;
            }
        }
        for el in self.post_commit_actions.iter() {
            el.collect_unique_accounts(container)?;
        }
        for el in self.post_undelegate_actions.iter() {
            el.collect_unique_accounts(container)?;
        }
        Ok(())
    }

    pub(in crate::intent_bundle) fn into_args(
        self,
        indices_map: &[&Address],
    ) -> Result<CommitAndUndelegateArgs<'args>, ProgramError> {
        // Build account indices
        let mut indices = NoVec::<u8, MAX_STATIC_CPI_ACCOUNTS>::new();
        for account in self.accounts {
            let idx =
                get_index(indices_map, account.address()).ok_or(ProgramError::InvalidArgument)?;
            indices.try_push(idx)?;
        }

        // Build commit type
        let commit_type = if self.post_commit_actions.is_empty() {
            CommitTypeArgs::Standalone(indices)
        } else {
            let mut base_actions = NoVec::<_, MAX_ACTIONS_NUM>::new();
            for handler in self.post_commit_actions {
                base_actions.try_push(handler.args(indices_map)?)?;
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
                base_actions.try_push(handler.args(indices_map)?)?;
            }
            UndelegateTypeArgs::WithBaseActions { base_actions }
        };

        Ok(CommitAndUndelegateArgs {
            commit_type,
            undelegate_type,
        })
    }

    pub(crate) fn get_actions_len(&self) -> usize {
        self.post_commit_actions.len() + self.post_undelegate_actions.len()
    }

    pub(crate) fn get_action_callback(
        &self,
        mut action_index: usize,
    ) -> Option<&ActionCallback<'args>> {
        let post_commit_len = self.post_commit_actions.len();
        if action_index < post_commit_len {
            return self
                .post_commit_actions
                .get(action_index)
                .and_then(|el| el.callback.as_ref());
        }
        action_index -= post_commit_len;

        if action_index < self.post_undelegate_actions.len() {
            self.post_undelegate_actions
                .get(action_index)
                .and_then(|el| el.callback.as_ref())
        } else {
            None
        }
    }
}

// ---------------------------------------------------------------------------
// BaseAction type
// ---------------------------------------------------------------------------

#[derive(Clone)]
pub struct CallHandler<'args> {
    pub destination_program: Address,
    pub escrow_authority: AccountView,
    pub args: ActionArgs<'args>,
    pub compute_units: u32,
    pub accounts: &'args [ShortAccountMeta],
    pub callback: Option<ActionCallback<'args>>,
}

impl<'args> CallHandler<'args> {
    #[inline(never)]
    pub(super) fn collect_unique_accounts(
        &self,
        container: &mut NoVec<AccountView, MAX_STATIC_CPI_ACCOUNTS>,
    ) -> ProgramResult {
        if !container.contains(&self.escrow_authority) {
            container.try_push(self.escrow_authority.clone())?;
        }
        Ok(())
    }

    #[allow(clippy::clone_on_copy)]
    pub(crate) fn args(
        &self,
        indices_map: &[&Address],
    ) -> Result<BaseActionArgs<'args>, ProgramError> {
        let escrow_authority_index = get_index(indices_map, self.escrow_authority.address())
            .ok_or(ProgramError::InvalidArgument)?;
        Ok(BaseActionArgs {
            args: self.args.clone(),
            compute_units: self.compute_units,
            destination_program: self.destination_program.clone(),
            escrow_authority: escrow_authority_index,
            accounts: self.accounts,
        })
    }
}

// ---------------------------------------------------------------------------
// Callback type
// ---------------------------------------------------------------------------

/// Callback to invoke after a specific action is executed on the base layer.
#[derive(Clone)]
pub struct ActionCallback<'args> {
    pub destination_program: Address,
    pub discriminator: &'args [u8],
    pub payload: &'args [u8],
    pub compute_units: u32,
    pub accounts: &'args [ShortAccountMeta],
}

impl<'args> ActionCallback<'args> {
    #[allow(clippy::clone_on_copy)]
    pub(super) fn args(
        &self,
        action_index: u8,
    ) -> Result<AddActionCallbackArgs<'args>, ProgramError> {
        Ok(AddActionCallbackArgs {
            action_index,
            destination_program: self.destination_program.clone(),
            discriminator: self.discriminator,
            payload: self.payload,
            compute_units: self.compute_units,
            accounts: self.accounts,
        })
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Gets the index of a pubkey in the deduplicated pubkey list.
/// Returns None if the pubkey is not found.
pub(in crate::intent_bundle) fn get_index(pubkeys: &[&Address], needle: &Address) -> Option<u8> {
    pubkeys.iter().position(|k| k == &needle).map(|i| i as u8)
}
