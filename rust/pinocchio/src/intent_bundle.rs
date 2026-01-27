//! Intent Bundle module for pinocchio
//!
//! This module provides builders for creating `MagicIntentBundle` instructions
//! that can be used to schedule commits, undelegates, and base-layer actions
//! through the Magic program.

#[cfg(not(feature = "std"))]
use alloc::{vec, vec::Vec};
#[cfg(feature = "std")]
use std::{vec, vec::Vec};

use core::mem::MaybeUninit;
use pinocchio::{
    account_info::AccountInfo,
    cpi::{slice_invoke, MAX_CPI_ACCOUNTS},
    instruction::{AccountMeta, Instruction},
    program_error::ProgramError,
    pubkey::Pubkey,
    ProgramResult,
};

// ---------------------------------------------------------
// Args types for serialization
// ---------------------------------------------------------

/// Action arguments containing escrow index and instruction data.
pub struct ActionArgs {
    pub escrow_index: u8,
    pub data: Vec<u8>,
}

impl ActionArgs {
    pub fn new(data: Vec<u8>) -> Self {
        Self {
            escrow_index: 255,
            data,
        }
    }

    pub fn with_escrow_index(mut self, index: u8) -> Self {
        self.escrow_index = index;
        self
    }

    fn serialize(&self, buf: &mut Vec<u8>) {
        buf.push(self.escrow_index);
        buf.extend_from_slice(&(self.data.len() as u64).to_le_bytes());
        buf.extend_from_slice(&self.data);
    }
}

/// A compact account meta (pubkey + is_writable).
pub struct ShortAccountMeta {
    pub pubkey: Pubkey,
    pub is_writable: bool,
}

impl ShortAccountMeta {
    fn serialize(&self, buf: &mut Vec<u8>) {
        buf.extend_from_slice(self.pubkey.as_ref());
        buf.push(self.is_writable as u8);
    }
}

/// Base action arguments for serialization.
pub struct BaseActionArgs {
    pub args: ActionArgs,
    pub compute_units: u32,
    pub escrow_authority: u8,
    pub destination_program: Pubkey,
    pub accounts: Vec<ShortAccountMeta>,
}

impl BaseActionArgs {
    fn serialize(&self, buf: &mut Vec<u8>) {
        self.args.serialize(buf);
        buf.extend_from_slice(&self.compute_units.to_le_bytes());
        buf.push(self.escrow_authority);
        buf.extend_from_slice(self.destination_program.as_ref());
        buf.extend_from_slice(&(self.accounts.len() as u64).to_le_bytes());
        for account in &self.accounts {
            account.serialize(buf);
        }
    }
}

/// Commit type arguments for serialization.
pub enum CommitTypeArgs {
    Standalone(Vec<u8>),
    WithBaseActions {
        committed_accounts: Vec<u8>,
        base_actions: Vec<BaseActionArgs>,
    },
}

impl CommitTypeArgs {
    fn serialize(&self, buf: &mut Vec<u8>) {
        match self {
            Self::Standalone(indices) => {
                buf.extend_from_slice(&0u32.to_le_bytes()); // variant 0
                buf.extend_from_slice(&(indices.len() as u64).to_le_bytes());
                buf.extend_from_slice(indices);
            }
            Self::WithBaseActions {
                committed_accounts,
                base_actions,
            } => {
                buf.extend_from_slice(&1u32.to_le_bytes()); // variant 1
                buf.extend_from_slice(&(committed_accounts.len() as u64).to_le_bytes());
                buf.extend_from_slice(committed_accounts);
                buf.extend_from_slice(&(base_actions.len() as u64).to_le_bytes());
                for action in base_actions {
                    action.serialize(buf);
                }
            }
        }
    }
}

/// Undelegate type arguments for serialization.
pub enum UndelegateTypeArgs {
    Standalone,
    WithBaseActions { base_actions: Vec<BaseActionArgs> },
}

impl UndelegateTypeArgs {
    fn serialize(&self, buf: &mut Vec<u8>) {
        match self {
            Self::Standalone => {
                buf.extend_from_slice(&0u32.to_le_bytes()); // variant 0
            }
            Self::WithBaseActions { base_actions } => {
                buf.extend_from_slice(&1u32.to_le_bytes()); // variant 1
                buf.extend_from_slice(&(base_actions.len() as u64).to_le_bytes());
                for action in base_actions {
                    action.serialize(buf);
                }
            }
        }
    }
}

/// Commit and undelegate arguments for serialization.
pub struct CommitAndUndelegateArgs {
    pub commit_type: CommitTypeArgs,
    pub undelegate_type: UndelegateTypeArgs,
}

impl CommitAndUndelegateArgs {
    fn serialize(&self, buf: &mut Vec<u8>) {
        self.commit_type.serialize(buf);
        self.undelegate_type.serialize(buf);
    }
}

/// Magic intent bundle arguments for serialization.
pub struct MagicIntentBundleArgs {
    pub commit: Option<CommitTypeArgs>,
    pub commit_and_undelegate: Option<CommitAndUndelegateArgs>,
    pub standalone_actions: Vec<BaseActionArgs>,
}

impl MagicIntentBundleArgs {
    fn serialize(&self, buf: &mut Vec<u8>) {
        // Option<CommitTypeArgs>
        match &self.commit {
            None => buf.push(0),
            Some(commit) => {
                buf.push(1);
                commit.serialize(buf);
            }
        }
        // Option<CommitAndUndelegateArgs>
        match &self.commit_and_undelegate {
            None => buf.push(0),
            Some(cau) => {
                buf.push(1);
                cau.serialize(buf);
            }
        }
        // Vec<BaseActionArgs>
        buf.extend_from_slice(&(self.standalone_actions.len() as u64).to_le_bytes());
        for action in &self.standalone_actions {
            action.serialize(buf);
        }
    }
}

/// MagicBlockInstruction variant discriminant for ScheduleIntentBundle
const SCHEDULE_INTENT_BUNDLE_DISCRIMINANT: u32 = 6;

/// Serializes the ScheduleIntentBundle instruction data.
fn serialize_schedule_intent_bundle(args: &MagicIntentBundleArgs) -> Vec<u8> {
    let mut buf = Vec::new();
    buf.extend_from_slice(&SCHEDULE_INTENT_BUNDLE_DISCRIMINANT.to_le_bytes());
    args.serialize(&mut buf);
    buf
}

// ---------------------------------------------------------
// Helper functions
// ---------------------------------------------------------

/// Gets the index of a pubkey in the deduplicated pubkey list.
/// Returns None if the pubkey is not found.
fn get_index(pubkeys: &[Pubkey], needle: &Pubkey) -> Option<u8> {
    pubkeys
        .iter()
        .position(|k| k == needle)
        .map(|i| i as u8)
}

/// Removes duplicates from array by pubkey.
/// Returns the list of unique pubkeys (index in list = account index).
fn filter_duplicates(container: &mut Vec<&AccountInfo>) -> Vec<Pubkey> {
    let mut seen: Vec<Pubkey> = Vec::new();
    container.retain(|el| {
        if seen.contains(el.key()) {
            false
        } else {
            seen.push(*el.key());
            true
        }
    });
    seen
}

// ---------------------------------------------------------
// Types
// ---------------------------------------------------------

/// Intent to be scheduled for execution on the base layer.
pub enum MagicIntent<'a> {
    /// Standalone actions to execute on base layer without commit/undelegate semantics.
    StandaloneActions(Vec<BaseAction<'a>>),
    /// Commit accounts to base layer, optionally with post-commit actions.
    Commit(CommitType<'a>),
    /// Commit accounts and undelegate them, optionally with post-commit and post-undelegate actions.
    CommitAndUndelegate(CommitAndUndelegate<'a>),
}

/// Type of commit, can be standalone or with post-commit actions.
pub enum CommitType<'a> {
    /// Regular commit without actions
    Standalone(Vec<&'a AccountInfo>),
    /// Commits accounts and runs actions
    WithHandler {
        committed_accounts: Vec<&'a AccountInfo>,
        call_handlers: Vec<BaseAction<'a>>,
    },
}

impl<'a> CommitType<'a> {
    pub fn committed_accounts(&self) -> &Vec<&'a AccountInfo> {
        match self {
            Self::Standalone(accounts) => accounts,
            Self::WithHandler {
                committed_accounts, ..
            } => committed_accounts,
        }
    }

    fn committed_accounts_mut(&mut self) -> &mut Vec<&'a AccountInfo> {
        match self {
            Self::Standalone(accounts) => accounts,
            Self::WithHandler {
                committed_accounts, ..
            } => committed_accounts,
        }
    }

    fn dedup(&mut self) -> Vec<Pubkey> {
        let committed_accounts = self.committed_accounts_mut();
        let mut seen = Vec::with_capacity(committed_accounts.len());
        committed_accounts.retain(|el| {
            if seen.contains(el.key()) {
                false
            } else {
                seen.push(*el.key());
                true
            }
        });
        seen
    }

    fn collect_accounts(&self, container: &mut Vec<&'a AccountInfo>) {
        match self {
            Self::Standalone(accounts) => container.extend(accounts.iter().copied()),
            Self::WithHandler {
                committed_accounts,
                call_handlers,
            } => {
                container.extend(committed_accounts.iter().copied());
                for handler in call_handlers {
                    handler.collect_accounts(container);
                }
            }
        }
    }

    fn into_args(self, pubkeys: &[Pubkey]) -> Result<CommitTypeArgs, ProgramError> {
        match self {
            Self::Standalone(accounts) => {
                let mut indices = Vec::with_capacity(accounts.len());
                for account in accounts {
                    let idx = get_index(pubkeys, account.key())
                        .ok_or(ProgramError::InvalidAccountData)?;
                    indices.push(idx);
                }
                Ok(CommitTypeArgs::Standalone(indices))
            }
            Self::WithHandler {
                committed_accounts,
                call_handlers,
            } => {
                let mut committed_indices = Vec::with_capacity(committed_accounts.len());
                for account in committed_accounts {
                    let idx = get_index(pubkeys, account.key())
                        .ok_or(ProgramError::InvalidAccountData)?;
                    committed_indices.push(idx);
                }
                let mut base_actions = Vec::with_capacity(call_handlers.len());
                for handler in call_handlers {
                    base_actions.push(handler.into_args(pubkeys)?);
                }
                Ok(CommitTypeArgs::WithBaseActions {
                    committed_accounts: committed_indices,
                    base_actions,
                })
            }
        }
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

/// Type of undelegate, can be standalone or with post-undelegate actions.
pub enum UndelegateType<'a> {
    Standalone,
    WithHandler(Vec<BaseAction<'a>>),
}

impl<'a> UndelegateType<'a> {
    fn collect_accounts(&self, container: &mut Vec<&'a AccountInfo>) {
        match self {
            Self::Standalone => {}
            Self::WithHandler(handlers) => {
                for handler in handlers {
                    handler.collect_accounts(container);
                }
            }
        }
    }

    fn into_args(self, pubkeys: &[Pubkey]) -> Result<UndelegateTypeArgs, ProgramError> {
        match self {
            Self::Standalone => Ok(UndelegateTypeArgs::Standalone),
            Self::WithHandler(handlers) => {
                let mut base_actions = Vec::with_capacity(handlers.len());
                for handler in handlers {
                    base_actions.push(handler.into_args(pubkeys)?);
                }
                Ok(UndelegateTypeArgs::WithBaseActions { base_actions })
            }
        }
    }
}

/// Commit and undelegate intent.
pub struct CommitAndUndelegate<'a> {
    pub commit_type: CommitType<'a>,
    pub undelegate_type: UndelegateType<'a>,
}

impl<'a> CommitAndUndelegate<'a> {
    fn collect_accounts(&self, container: &mut Vec<&'a AccountInfo>) {
        self.commit_type.collect_accounts(container);
        self.undelegate_type.collect_accounts(container);
    }

    fn into_args(
        self,
        pubkeys: &[Pubkey],
    ) -> Result<CommitAndUndelegateArgs, ProgramError> {
        let commit_type = self.commit_type.into_args(pubkeys)?;
        let undelegate_type = self.undelegate_type.into_args(pubkeys)?;
        Ok(CommitAndUndelegateArgs {
            commit_type,
            undelegate_type,
        })
    }

    fn dedup(&mut self) -> Vec<Pubkey> {
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

/// Base action to execute on the base layer.
pub struct BaseAction<'a> {
    /// Instruction data to pass to the destination program
    pub args: ActionArgs,
    /// Compute units this action will use
    pub compute_units: u32,
    /// Account authorizing action on actor PDA
    pub escrow_authority: &'a AccountInfo,
    /// Address of destination program
    pub destination_program: Pubkey,
    /// Accounts to include in the action (pubkey + is_writable)
    pub accounts: Vec<ShortAccountMeta>,
}

impl<'a> BaseAction<'a> {
    fn collect_accounts(&self, container: &mut Vec<&'a AccountInfo>) {
        container.push(self.escrow_authority);
    }

    fn into_args(self, pubkeys: &[Pubkey]) -> Result<BaseActionArgs, ProgramError> {
        let escrow_authority = get_index(pubkeys, self.escrow_authority.key())
            .ok_or(ProgramError::InvalidAccountData)?;
        Ok(BaseActionArgs {
            args: self.args,
            compute_units: self.compute_units,
            escrow_authority,
            destination_program: self.destination_program,
            accounts: self.accounts,
        })
    }
}

// ---------------------------------------------------------
// Builders
// ---------------------------------------------------------

/// Builder for Commit Intent.
pub struct CommitIntentBuilder<'a> {
    accounts: &'a [&'a AccountInfo],
    actions: Vec<BaseAction<'a>>,
}

impl<'a> CommitIntentBuilder<'a> {
    pub fn new(accounts: &'a [&'a AccountInfo]) -> Self {
        Self {
            accounts,
            actions: vec![],
        }
    }

    pub fn add_post_commit_actions(
        mut self,
        actions: impl IntoIterator<Item = BaseAction<'a>>,
    ) -> Self {
        self.actions.extend(actions);
        self
    }

    /// Builds and returns Commit Intent Type
    pub fn build(self) -> CommitType<'a> {
        let committed_accounts = self.accounts.to_vec();
        if self.actions.is_empty() {
            CommitType::Standalone(committed_accounts)
        } else {
            CommitType::WithHandler {
                committed_accounts,
                call_handlers: self.actions,
            }
        }
    }
}

/// Builder for CommitAndUndelegate Intent.
pub struct CommitAndUndelegateIntentBuilder<'a> {
    accounts: &'a [&'a AccountInfo],
    post_commit_actions: Vec<BaseAction<'a>>,
    post_undelegate_actions: Vec<BaseAction<'a>>,
}

impl<'a> CommitAndUndelegateIntentBuilder<'a> {
    pub fn new(accounts: &'a [&'a AccountInfo]) -> Self {
        Self {
            accounts,
            post_commit_actions: vec![],
            post_undelegate_actions: vec![],
        }
    }

    pub fn add_post_commit_actions(
        mut self,
        actions: impl IntoIterator<Item = BaseAction<'a>>,
    ) -> Self {
        self.post_commit_actions.extend(actions);
        self
    }

    pub fn add_post_undelegate_actions(
        mut self,
        actions: impl IntoIterator<Item = BaseAction<'a>>,
    ) -> Self {
        self.post_undelegate_actions.extend(actions);
        self
    }

    pub fn build(self) -> CommitAndUndelegate<'a> {
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

// ---------------------------------------------------------
// Intent Bundle
// ---------------------------------------------------------

/// Bundle of intents.
#[derive(Default)]
struct MagicIntentBundle<'a> {
    standalone_actions: Vec<BaseAction<'a>>,
    commit_intent: Option<CommitType<'a>>,
    commit_and_undelegate_intent: Option<CommitAndUndelegate<'a>>,
}

impl<'a> MagicIntentBundle<'a> {
    fn add_intent(&mut self, intent: MagicIntent<'a>) {
        match intent {
            MagicIntent::StandaloneActions(value) => self.standalone_actions.extend(value),
            MagicIntent::Commit(value) => {
                if let Some(ref mut commit) = self.commit_intent {
                    commit.merge(value);
                } else {
                    self.commit_intent = Some(value);
                }
            }
            MagicIntent::CommitAndUndelegate(value) => {
                if let Some(ref mut cau) = self.commit_and_undelegate_intent {
                    cau.merge(value);
                } else {
                    self.commit_and_undelegate_intent = Some(value);
                }
            }
        }
    }

    fn into_args(
        self,
        pubkeys: &[Pubkey],
    ) -> Result<MagicIntentBundleArgs, ProgramError> {
        let commit = self
            .commit_intent
            .map(|c| c.into_args(pubkeys))
            .transpose()?;
        let commit_and_undelegate = self
            .commit_and_undelegate_intent
            .map(|c| c.into_args(pubkeys))
            .transpose()?;
        let mut standalone_actions = Vec::with_capacity(self.standalone_actions.len());
        for action in self.standalone_actions {
            standalone_actions.push(action.into_args(pubkeys)?);
        }

        Ok(MagicIntentBundleArgs {
            commit,
            commit_and_undelegate,
            standalone_actions,
        })
    }

    fn collect_accounts(&self, all_accounts: &mut Vec<&'a AccountInfo>) {
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

    fn normalize(&mut self) {
        if let Some(ref mut value) = self.commit_intent {
            value.dedup();
        }
        let cau = self.commit_and_undelegate_intent.as_mut().map(|value| {
            let seen = value.dedup();
            (seen, value)
        });

        let (mut commit, cau, cau_pubkeys) = match (self.commit_intent.take(), cau) {
            (Some(commit), Some((cau_pubkeys, cau))) => (commit, cau, cau_pubkeys),
            (Some(commit), None) => {
                self.commit_intent = Some(commit);
                return;
            }
            _ => return,
        };

        commit
            .committed_accounts_mut()
            .retain(|el| !cau_pubkeys.contains(el.key()));

        if commit.committed_accounts().is_empty() {
            cau.commit_type.merge(commit);
        } else {
            self.commit_intent = Some(commit);
        }
    }
}

// ---------------------------------------------------------
// MagicIntentBundleBuilder
// ---------------------------------------------------------

/// Builds a `ScheduleIntentBundle` instruction by aggregating multiple intents.
pub struct MagicIntentBundleBuilder<'a> {
    payer: &'a AccountInfo,
    magic_context: &'a AccountInfo,
    magic_program: &'a AccountInfo,
    intent_bundle: MagicIntentBundle<'a>,
}

impl<'a> MagicIntentBundleBuilder<'a> {
    pub fn new(
        payer: &'a AccountInfo,
        magic_context: &'a AccountInfo,
        magic_program: &'a AccountInfo,
    ) -> Self {
        Self {
            payer,
            magic_context,
            magic_program,
            intent_bundle: MagicIntentBundle::default(),
        }
    }

    /// Adds an intent to the bundle.
    pub fn add_intent(mut self, intent: MagicIntent<'a>) -> Self {
        self.intent_bundle.add_intent(intent);
        self
    }

    /// Adds (or merges) a `Commit` intent into the bundle.
    pub fn add_commit(mut self, commit: CommitType<'a>) -> Self {
        self.intent_bundle.add_intent(MagicIntent::Commit(commit));
        self
    }

    /// Adds (or merges) a `CommitAndUndelegate` intent into the bundle.
    pub fn add_commit_and_undelegate(mut self, value: CommitAndUndelegate<'a>) -> Self {
        self.intent_bundle
            .add_intent(MagicIntent::CommitAndUndelegate(value));
        self
    }

    /// Adds standalone base-layer actions.
    pub fn add_standalone_actions(
        mut self,
        actions: impl IntoIterator<Item = BaseAction<'a>>,
    ) -> Self {
        self.intent_bundle
            .add_intent(MagicIntent::StandaloneActions(actions.into_iter().collect()));
        self
    }

    /// Builds the deduplicated account list and instruction data.
    pub fn build(mut self) -> Result<(Vec<&'a AccountInfo>, Vec<u8>), ProgramError> {
        self.intent_bundle.normalize();

        let mut all_accounts = vec![self.payer, self.magic_context];
        self.intent_bundle.collect_accounts(&mut all_accounts);

        let pubkeys = filter_duplicates(&mut all_accounts);
        let args = self.intent_bundle.into_args(&pubkeys)?;
        let data = serialize_schedule_intent_bundle(&args);

        Ok((all_accounts, data))
    }

    /// Builds the instruction and immediately invokes it.
    pub fn build_and_invoke(self) -> ProgramResult {
        let magic_program_key = self.magic_program.key();
        let (accounts, data) = self.build()?;

        let num_accounts = accounts.len();
        if num_accounts > MAX_CPI_ACCOUNTS {
            return Err(ProgramError::InvalidArgument);
        }

        const UNINIT_META: MaybeUninit<AccountMeta> = MaybeUninit::<AccountMeta>::uninit();
        let mut account_metas = [UNINIT_META; MAX_CPI_ACCOUNTS];

        for (i, account) in accounts.iter().enumerate() {
            unsafe {
                account_metas.get_unchecked_mut(i).write(AccountMeta::new(
                    account.key(),
                    account.is_writable(),
                    account.is_signer(),
                ));
            }
        }

        let ix = Instruction {
            program_id: magic_program_key,
            accounts: unsafe {
                core::slice::from_raw_parts(
                    account_metas.as_ptr() as *const AccountMeta,
                    num_accounts,
                )
            },
            data: &data,
        };

        slice_invoke(&ix, &accounts)
    }
}
