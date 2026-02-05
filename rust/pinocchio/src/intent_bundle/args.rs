use crate::intent_bundle::no_vec::NoVec;
use core::mem::MaybeUninit;
use pinocchio::{AccountView, ProgramResult};
use serde::{Deserialize, Serialize};
use solana_address::Address;
// ---------------------------------------------------------
// Args types for serialization
// ---------------------------------------------------------

const MAX_ACTIONS_NUM: usize = 10u8 as usize;
const MAX_COMMITTED_ACCOUNTS_NUM: usize = 64u8 as usize;
const MAX_ACCOUNTS: usize = u8::MAX as usize;

/// Action arguments containing escrow index and instruction data.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ActionArgs<'a> {
    pub escrow_index: u8,
    pub data: &'a [u8],
}

impl<'a> ActionArgs<'a> {
    pub fn new(data: &'a [u8]) -> Self {
        Self {
            escrow_index: 255,
            data,
        }
    }

    pub fn escrow_index(&self) -> u8 {
        self.escrow_index
    }

    pub fn data(&self) -> &'a [u8] {
        &self.data
    }

    pub fn with_escrow_index(mut self, index: u8) -> Self {
        self.escrow_index = index;
        self
    }
}

/// Base action arguments for serialization.
#[derive(Clone, Serialize, Debug)]
pub struct BaseActionArgs<'args> {
    pub args: ActionArgs<'args>,
    pub compute_units: u32,
    pub escrow_authority: u8,
    pub destination_program: Address,
    pub accounts: NoVec<ShortAccountMeta, MAX_ACTIONS_NUM>,
}

/// A compact account meta used for base-layer actions.
///
/// Unlike `solana_instruction::AccountMeta`, this type **does not** carry an
/// `is_signer` flag. Users cannot request signatures: the only signer available
/// is the validator.
#[derive(Debug, Default, Clone, Serialize)]
pub struct ShortAccountMeta {
    pub pubkey: Address,
    pub is_writable: bool,
}

/// Commit type arguments for serialization.
#[derive(Serialize)]
pub enum CommitTypeArgs<'args> {
    // we generate it
    Standalone(NoVec<u8, MAX_COMMITTED_ACCOUNTS_NUM>), // slice or NoVec
    WithBaseActions {
        committed_accounts: NoVec<u8, MAX_COMMITTED_ACCOUNTS_NUM>,
        base_actions: NoVec<BaseActionArgs<'args>, MAX_ACTIONS_NUM>,
    },
}

/// Undelegate type arguments for serialization.
#[derive(Serialize)]
pub enum UndelegateTypeArgs<'args> {
    Standalone,
    WithBaseActions { base_actions: NoVec<BaseActionArgs<'args>, MAX_ACTIONS_NUM> },
}

/// Commit and undelegate arguments for serialization.
#[derive(Serialize)]
pub struct CommitAndUndelegateArgs<'args> {
    pub commit_type: CommitTypeArgs<'args>,
    pub undelegate_type: UndelegateTypeArgs<'args>,
}

/// Magic intent bundle arguments for serialization.
#[derive(Serialize)]
pub struct MagicIntentBundleArgs<'args> {
    pub commit: Option<CommitTypeArgs<'args>>,
    pub commit_and_undelegate: Option<CommitAndUndelegateArgs<'args>>,
    pub standalone_actions: NoVec<BaseActionArgs<'args>, MAX_ACTIONS_NUM>,
}

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
