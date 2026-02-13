use crate::intent_bundle::args::{
    ActionArgs, BaseActionArgs, CommitAndUndelegateArgs, CommitTypeArgs, MagicIntentBundleArgs,
    ShortAccountMeta, UndelegateTypeArgs,
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
pub enum MagicIntent<'a, 'args> {
    /// Standalone actions to execute on base layer without commit/undelegate semantics.
    StandaloneActions(&'a [CallHandler<'args>]),
    /// Commit accounts to base layer, optionally with post-commit actions.
    Commit(CommitIntent<'a, 'args>),
    /// Commit accounts and undelegate them, optionally with post-commit and post-undelegate actions.
    CommitAndUndelegate(CommitAndUndelegateIntent<'a, 'args>),
}

/// Bundle of Intents
///
/// Note: if `CommitIntent` & `CommitAndUndelegateIntent` has an account overlap
/// they will be undelegated
///
/// Intents assumed to be independent and self-sufficient,
/// hence order in which they were inserted doesn't matter
#[derive(Default)]
pub(super) struct MagicIntentBundle<'a, 'args> {
    pub(super) standalone_actions: &'a [CallHandler<'args>],
    pub(super) commit_intent: Option<CommitIntent<'a, 'args>>,
    pub(super) commit_and_undelegate_intent: Option<CommitAndUndelegateIntent<'a, 'args>>,
}

impl<'a, 'args> MagicIntentBundle<'a, 'args> {
    /// Inserts an intent into the bundle, merging with any existing intent of the same category.
    pub(super) fn add_intent(&mut self, intent: MagicIntent<'a, 'args>) {
        match intent {
            MagicIntent::StandaloneActions(value) => {
                self.standalone_actions = value;
            }
            MagicIntent::Commit(value) => {
                self.commit_intent = Some(value);
            }
            MagicIntent::CommitAndUndelegate(value) => {
                self.commit_and_undelegate_intent = Some(value);
            }
        }
    }

    /// Consumes the bundle and encodes it into `MagicIntentBundleArgs` using an indices map.
    ///
    /// `indices_map` is a natural map: `indices_map[i]` is the address at index `i`.
    pub(super) fn into_args(
        self,
        indices_map: &[Address],
    ) -> Result<MagicIntentBundleArgs<'args>, ProgramError> {
        let commit = self
            .commit_intent
            .map(|c| c.into_args(indices_map))
            .transpose()?;
        let commit_and_undelegate = self
            .commit_and_undelegate_intent
            .map(|c| c.into_args(indices_map))
            .transpose()?;
        let mut standalone_actions = NoVec::<BaseActionArgs<'args>, MAX_ACTIONS_NUM>::new();
        for ch in self.standalone_actions {
            standalone_actions.try_push(ch.args(indices_map)?)?;
        }

        Ok(MagicIntentBundleArgs {
            commit,
            commit_and_undelegate,
            standalone_actions,
        })
    }

    /// Collects all accounts referenced by intents in this bundle.
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

    /// Validates that all present intents have at least one committed account.
    pub(super) fn validate(&self) -> ProgramResult {
        if let Some(ref commit) = self.commit_intent {
            commit.validate()?;
        }
        if let Some(ref cau) = self.commit_and_undelegate_intent {
            cau.validate()?;
        }
        Ok(())
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
    pub(super) fn normalize(&mut self) -> ProgramResult {
        let cau_seen = self
            .commit_and_undelegate_intent
            .as_mut()
            .map(|cau| cau.dedup())
            .transpose()?;
        let Some(mut commit) = self.commit_intent.take() else {
            return Ok(()); // No commit intent, nothing more to normalize
        };

        let mut seen = cau_seen.unwrap_or_default();
        commit.dedup(&mut seen)?;

        // If commit lost all its accounts to CAU overlap, move its actions
        // into CAU's post-commit actions and drop the empty commit intent.
        if commit.accounts.is_empty() {
            Err(ProgramError::InvalidArgument)
        } else {
            self.commit_intent = Some(commit);
            Ok(())
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
    accounts: NoVec<ShortAccountMeta, MAX_STATIC_CPI_ACCOUNTS>,
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

    pub fn add_accounts_slice(&mut self, accounts: &[ShortAccountMeta]) -> ProgramResult {
        self.accounts.try_append_slice(accounts)?;
        Ok(())
    }

    pub fn add_accounts<const N: usize>(
        &mut self,
        accounts: [ShortAccountMeta; N],
    ) -> ProgramResult {
        self.accounts.try_append(accounts)?;
        Ok(())
    }

    pub(super) fn collect_unique_accounts(
        &self,
        container: &mut NoVec<AccountView, MAX_STATIC_CPI_ACCOUNTS>,
    ) -> ProgramResult {
        if !container.contains(&self.escrow_authority) {
            container.try_push(self.escrow_authority.clone())?;
        }
        Ok(())
    }

    pub(crate) fn args(
        &self,
        indices_map: &[Address],
    ) -> Result<BaseActionArgs<'args>, ProgramError> {
        let escrow_authority_index = get_index(indices_map, self.escrow_authority.address())
            .ok_or(ProgramError::InvalidArgument)?;
        Ok(BaseActionArgs {
            args: self.args.clone(),
            compute_units: self.compute_units,
            destination_program: self.destination_program.clone(),
            escrow_authority: escrow_authority_index,
            accounts: self.accounts.clone(),
        })
    }
}

pub struct CommitIntent<'a, 'args> {
    pub(super) accounts: NoVec<AccountView, MAX_STATIC_CPI_ACCOUNTS>,
    pub(super) actions: &'a [CallHandler<'args>],
}

impl<'a, 'args> CommitIntent<'a, 'args> {
    /// Validates that this commit intent has at least one account to commit.
    fn validate(&self) -> ProgramResult {
        if self.accounts.is_empty() {
            Err(ProgramError::InvalidInstructionData)
        } else {
            Ok(())
        }
    }

    /// Deduplicates committed accounts by address. Accounts whose address is
    /// already in `seen` are removed. Newly seen addresses are added to `seen`.
    fn dedup(&mut self, seen: &mut NoVec<Address, MAX_STATIC_CPI_ACCOUNTS>) -> ProgramResult {
        dedup_accounts(&mut self.accounts, seen)
    }

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

    fn into_args(self, indices_map: &[Address]) -> Result<CommitTypeArgs<'args>, ProgramError> {
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
}

pub struct CommitAndUndelegateIntent<'a, 'args> {
    pub(super) accounts: NoVec<AccountView, MAX_STATIC_CPI_ACCOUNTS>,
    pub(super) post_commit_actions: &'a [CallHandler<'args>],
    pub(super) post_undelegate_actions: &'a [CallHandler<'args>],
}

impl<'a, 'args> CommitAndUndelegateIntent<'a, 'args> {
    /// Validates that this commit-and-undelegate intent has at least one account.
    fn validate(&self) -> ProgramResult {
        if self.accounts.is_empty() {
            Err(ProgramError::InvalidInstructionData)
        } else {
            Ok(())
        }
    }

    /// Deduplicates committed accounts by address and returns the set of
    /// unique addresses (for cross-intent overlap detection).
    fn dedup(&mut self) -> Result<NoVec<Address, MAX_STATIC_CPI_ACCOUNTS>, ProgramError> {
        let mut seen = NoVec::<Address, MAX_STATIC_CPI_ACCOUNTS>::new();
        dedup_accounts(&mut self.accounts, &mut seen)?;
        Ok(seen)
    }

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

    fn into_args(
        self,
        indices_map: &[Address],
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
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Deduplicates `accounts` by address against a running `seen` set.
/// Accounts whose address is already in `seen` are removed; newly encountered
/// addresses are appended to `seen`.
fn dedup_accounts(
    accounts: &mut NoVec<AccountView, MAX_STATIC_CPI_ACCOUNTS>,
    seen: &mut NoVec<Address, MAX_STATIC_CPI_ACCOUNTS>,
) -> ProgramResult {
    let mut result: ProgramResult = Ok(());
    accounts.retain(|el| {
        if result.is_err() {
            return false;
        }
        let addr = el.address();
        if seen.contains(addr) {
            false
        } else {
            match seen.try_push(addr.clone()) {
                Ok(()) => true,
                Err(_) => {
                    result = Err(ProgramError::InvalidArgument);
                    false
                }
            }
        }
    });
    result
}

/// Gets the index of a pubkey in the deduplicated pubkey list.
/// Returns None if the pubkey is not found.
fn get_index(pubkeys: &[Address], needle: &Address) -> Option<u8> {
    pubkeys.iter().position(|k| k == needle).map(|i| i as u8)
}

#[cfg(test)]
mod size_tests {
    extern crate std;
    use super::*;
    use crate::intent_bundle::args::BaseActionArgs;

    #[test]
    fn print_sizes() {
        std::println!(
            "ShortAccountMeta: {}",
            core::mem::size_of::<ShortAccountMeta>()
        );
        std::println!(
            "NoVec<ShortAccountMeta, MAX_STATIC_CPI_ACCOUNTS>: {}",
            core::mem::size_of::<NoVec<ShortAccountMeta, MAX_STATIC_CPI_ACCOUNTS>>()
        );
        std::println!("CallHandler: {}", core::mem::size_of::<CallHandler>());
        std::println!("BaseActionArgs: {}", core::mem::size_of::<BaseActionArgs>());
        std::println!("CommitIntent: {}", core::mem::size_of::<CommitIntent>());
        std::println!(
            "CommitAndUndelegateIntent: {}",
            core::mem::size_of::<CommitAndUndelegateIntent>()
        );
        std::println!(
            "MagicIntentBundle: {}",
            core::mem::size_of::<MagicIntentBundle>()
        );
        std::println!("MAX_STATIC_CPI_ACCOUNTS: {}", MAX_STATIC_CPI_ACCOUNTS);
        std::println!("MAX_ACTIONS_NUM: {}", MAX_ACTIONS_NUM);
    }
}
