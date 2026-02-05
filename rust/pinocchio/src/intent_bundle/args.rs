use crate::intent_bundle::no_vec::NoVec;
use serde::{Serialize};
use solana_address::Address;
// ---------------------------------------------------------
// Args types for serialization
// ---------------------------------------------------------

const MAX_ACTIONS_NUM: usize = 10u8 as usize;
const MAX_COMMITTED_ACCOUNTS_NUM: usize = 64u8 as usize;
const MAX_ACCOUNTS: usize = pinocchio::cpi::MAX_CPI_ACCOUNTS;

/// Action arguments containing escrow index and instruction data.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, bincode::Encode)]
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
#[derive(Clone, Debug, Serialize, bincode::Encode)]
pub struct BaseActionArgs<'args> {
    pub args: ActionArgs<'args>,
    pub compute_units: u32,
    pub escrow_authority: u8,
    #[bincode(with_serde)]
    pub destination_program: Address,
    pub accounts: NoVec<ShortAccountMeta, MAX_ACTIONS_NUM>,
}

/// A compact account meta used for base-layer actions.
///
/// Unlike `solana_instruction::AccountMeta`, this type **does not** carry an
/// `is_signer` flag. Users cannot request signatures: the only signer available
/// is the validator.
#[derive(Debug, Default, Clone, Serialize, bincode::Encode)]
pub struct ShortAccountMeta {
    #[bincode(with_serde)]
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
    WithBaseActions {
        base_actions: NoVec<BaseActionArgs<'args>, MAX_ACTIONS_NUM>,
    },
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
