//! Streaming serialization wrappers for the intent bundle.
//!
//! These types implement [`bincode::Encode`] directly — without materializing the
//! large intermediate `MagicIntentBundleArgs` struct (~3488 bytes) — to stay within
//! the Solana sBPF per-function stack-frame limit of 4096 bytes.
//!
//! # Wire format
//!
//! All types produce output identical to the corresponding `*Args` types in
//! [`crate::intent_bundle::args`] when encoded with `bincode::config::legacy()`.

use crate::intent_bundle::args::{AddActionCallbackArgs, BaseActionArgs};
use crate::intent_bundle::types::{
    get_index, CallHandler, CommitAndUndelegateIntent, CommitIntent, MagicIntentBundle,
};
use crate::intent_bundle::ActionCallback;
use bincode::enc::Encoder;
use bincode::error::EncodeError;
use bincode::Encode;
use core::ops::Deref;
use pinocchio::error::ProgramError;
use solana_address::Address;

/// Bincode 1.x u32 LE discriminant for `MagicBlockInstruction::ScheduleIntentBundle` (variant index 11).
const SCHEDULE_INTENT_BUNDLE_DISCRIMINANT: [u8; 4] = 11u32.to_le_bytes();
/// Bincode 1.x u32 LE discriminant for `MagicBlockInstruction::AddActionCallback` (variant index 23).
const ADD_ACTION_CALLBACK_DISCRIMINANT: [u8; 4] = 23u32.to_le_bytes();
pub(crate) const DISCRIMINANT_SIZE: usize = SCHEDULE_INTENT_BUNDLE_DISCRIMINANT.len();

// ---------------------------------------------------------------------------
// Wrapper types
// ---------------------------------------------------------------------------

/// Streaming encoder for `CommitTypeArgs`.
///
/// `encode` calls [`CommitIntent::into_args`] which materializes a `CommitTypeArgs`
/// (~896 bytes) only within its own stack frame — not in `build_and_invoke`'s.
pub(super) struct CommitSerialize<'i, 'acc, 'args> {
    inner: CommitIntent<'acc, 'args>,
    indices_map: &'i [&'i Address],
}

impl<'i, 'acc, 'args> CommitSerialize<'i, 'acc, 'args> {
    pub(super) fn new(inner: CommitIntent<'acc, 'args>, indices_map: &'i [&'i Address]) -> Self {
        Self { inner, indices_map }
    }
}

impl<'acc, 'args> Deref for CommitSerialize<'_, 'acc, 'args> {
    type Target = CommitIntent<'acc, 'args>;
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl bincode::Encode for CommitSerialize<'_, '_, '_> {
    #[inline(never)]
    fn encode<E: Encoder>(&self, encoder: &mut E) -> Result<(), EncodeError> {
        self.inner // Copy — CommitIntent is two fat pointers
            .into_args(self.indices_map)
            .map_err(|_| EncodeError::Other("CommitIntent::into_args failed"))?
            .encode(encoder)
    }
}

/// Streaming encoder for `CommitAndUndelegateArgs`.
///
/// `encode` calls [`CommitAndUndelegateIntent::into_args`] which materializes a
/// `CommitAndUndelegateArgs` (~1712 bytes) only within its own stack frame.
pub(super) struct CommitAndUndelegateSerialize<'i, 'acc, 'args> {
    inner: CommitAndUndelegateIntent<'acc, 'args>,
    indices_map: &'i [&'i Address],
}

impl<'i, 'acc, 'args> CommitAndUndelegateSerialize<'i, 'acc, 'args> {
    pub(super) fn new(
        inner: CommitAndUndelegateIntent<'acc, 'args>,
        indices_map: &'i [&'i Address],
    ) -> Self {
        Self { inner, indices_map }
    }
}

impl<'acc, 'args> Deref for CommitAndUndelegateSerialize<'_, 'acc, 'args> {
    type Target = CommitAndUndelegateIntent<'acc, 'args>;
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl bincode::Encode for CommitAndUndelegateSerialize<'_, '_, '_> {
    #[inline(never)]
    fn encode<E: Encoder>(&self, encoder: &mut E) -> Result<(), EncodeError> {
        self.inner // Copy — CommitAndUndelegateIntent is three fat pointers
            .into_args(self.indices_map)
            .map_err(|_| EncodeError::Other("CommitAndUndelegateIntent::into_args failed"))?
            .encode(encoder)
    }
}

/// Streaming encoder for `MagicIntentBundleArgs`.
///
/// `Option<CommitSerialize>` and `Option<CommitAndUndelegateSerialize>` are encoded
/// by bincode's blanket `Option<T: Encode>` impl (`0u8` / `1u8 + value`), matching
/// the bincode 1.x wire format.
pub(super) struct MagicIntentBundleSerialize<'i, 'acc, 'args> {
    indices_map: &'i [&'i Address],
    commit: Option<CommitSerialize<'i, 'acc, 'args>>,
    commit_and_undelegate: Option<CommitAndUndelegateSerialize<'i, 'acc, 'args>>,
    /// Not yet implemented; always `None`. Reserved for wire-format compatibility.
    commit_finalize: Option<()>,
    /// Not yet implemented; always `None`. Reserved for wire-format compatibility.
    commit_finalize_and_undelegate: Option<()>,
    commit_finalize_compressed: Option<CommitSerialize<'i, 'acc, 'args>>,
    commit_finalize_compressed_and_undelegate:
        Option<CommitAndUndelegateSerialize<'i, 'acc, 'args>>,
    standalone_actions: &'args [CallHandler<'args>],
}

impl<'i, 'acc, 'args> MagicIntentBundleSerialize<'i, 'acc, 'args> {
    pub(super) fn new(
        indices_map: &'i [&'i Address],
        bundle: MagicIntentBundle<'acc, 'args>,
    ) -> Self {
        Self {
            commit: bundle
                .commit_intent
                .map(|c| CommitSerialize::new(c, indices_map)),
            commit_and_undelegate: bundle
                .commit_and_undelegate_intent
                .map(|c| CommitAndUndelegateSerialize::new(c, indices_map)),
            commit_finalize: None,
            commit_finalize_and_undelegate: None,
            commit_finalize_compressed: bundle
                .commit_finalize_compressed_intent
                .map(|c| CommitSerialize::new(c, indices_map)),
            commit_finalize_compressed_and_undelegate: bundle
                .commit_finalize_compressed_and_undelegate_intent
                .map(|c| CommitAndUndelegateSerialize::new(c, indices_map)),
            standalone_actions: bundle.standalone_actions,
            indices_map,
        }
    }

    pub(super) fn encode_intent_into_slice(
        &self,
        data_buf: &mut [u8],
    ) -> Result<usize, ProgramError> {
        const OFFSET: usize = SCHEDULE_INTENT_BUNDLE_DISCRIMINANT.len();

        data_buf[..OFFSET].copy_from_slice(&SCHEDULE_INTENT_BUNDLE_DISCRIMINANT);
        let len =
            bincode::encode_into_slice(self, &mut data_buf[OFFSET..], bincode::config::legacy())
                .map_err(|_| ProgramError::InvalidInstructionData)?;

        Ok(OFFSET + len)
    }

    pub(super) fn action_callback_iter(&self) -> ActionCallbackIter<'_, '_, '_, '_> {
        ActionCallbackIter::new(self)
    }

    pub(super) fn get_action_callback(
        &self,
        mut action_index: usize,
    ) -> Option<&ActionCallback<'args>> {
        if let Some(ref commit) = self.commit {
            let commit_action_len = commit.get_actions_len();
            if action_index < commit_action_len {
                return commit.get_action_callback(action_index);
            }
            action_index -= commit_action_len;
        }

        if let Some(ref commit_and_undelegate) = self.commit_and_undelegate {
            let cau_action_len = commit_and_undelegate.get_actions_len();
            if action_index < cau_action_len {
                return commit_and_undelegate.get_action_callback(action_index);
            }
            action_index -= cau_action_len;
        }

        if let Some(ref _commit_finalize) = self.commit_finalize {
            // TODO: implement once supported
        }

        if let Some(ref _commit_finalize_and_undelegate) = self.commit_finalize_and_undelegate {
            // TODO: implement once supported
        }

        if let Some(ref commit_finalize_compressed) = self.commit_finalize_compressed {
            let commit_action_len = commit_finalize_compressed.get_actions_len();
            if action_index < commit_action_len {
                return commit_finalize_compressed.get_action_callback(action_index);
            }
            action_index -= commit_action_len;
        }

        if let Some(ref commit_finalize_compressed_and_undelegate) =
            self.commit_finalize_compressed_and_undelegate
        {
            let cau_action_len = commit_finalize_compressed_and_undelegate.get_actions_len();
            if action_index < cau_action_len {
                return commit_finalize_compressed_and_undelegate.get_action_callback(action_index);
            }
            action_index -= cau_action_len;
        }

        let standalone_action_len = self.standalone_actions.len();
        if action_index < standalone_action_len {
            self.get_standalone_action_callback(action_index)
        } else {
            None
        }
    }

    #[allow(clippy::identity_op)]
    pub(super) fn get_actions_len(&self) -> usize {
        self.standalone_actions.len()
            + self
                .commit
                .as_ref()
                .map(|el| el.get_actions_len())
                .unwrap_or(0)
            + self
                .commit_and_undelegate
                .as_ref()
                .map(|el| el.get_actions_len())
                .unwrap_or(0)
            + 0 // TODO: support commit_finalize & commit_finalize_and_undelegate
            + 0 // TODO: support commit_finalize_and_undelegate
            + self
                .commit_finalize_compressed
                .as_ref()
                .map(|el| el.get_actions_len())
                .unwrap_or(0)
            + self
                .commit_finalize_compressed_and_undelegate
                .as_ref()
                .map(|el| el.get_actions_len())
                .unwrap_or(0)
    }

    pub(super) fn get_standalone_action_callback(
        &self,
        index: usize,
    ) -> Option<&ActionCallback<'args>> {
        self.standalone_actions
            .get(index)
            .and_then(|el| el.callback.as_ref())
    }
}

pub(super) struct ActionCallbackIter<'a, 'i, 'acc, 'args> {
    intent: &'a MagicIntentBundleSerialize<'i, 'acc, 'args>,
    intent_actions_num: usize,
    cur_action_index: usize,
}

impl<'a, 'i, 'acc, 'args> ActionCallbackIter<'a, 'i, 'acc, 'args> {
    pub(super) fn new(intent: &'a MagicIntentBundleSerialize<'i, 'acc, 'args>) -> Self {
        Self {
            intent,
            intent_actions_num: intent.get_actions_len(),
            cur_action_index: 0,
        }
    }
}

impl<'a, 'i, 'acc, 'args> Iterator for ActionCallbackIter<'a, 'i, 'acc, 'args> {
    type Item = (usize, &'a ActionCallback<'args>);
    fn next(&mut self) -> Option<Self::Item> {
        (self.cur_action_index..self.intent_actions_num).find_map(move |i| {
            self.cur_action_index = i + 1;
            self.intent
                .get_action_callback(i)
                .map(|el: &'a ActionCallback<'args>| (i, el))
        })
    }
}

impl bincode::Encode for MagicIntentBundleSerialize<'_, '_, '_> {
    #[inline(never)]
    fn encode<E: Encoder>(&self, encoder: &mut E) -> Result<(), EncodeError> {
        self.commit.encode(encoder)?;
        self.commit_and_undelegate.encode(encoder)?;
        self.commit_finalize.encode(encoder)?;
        self.commit_finalize_and_undelegate.encode(encoder)?;
        self.commit_finalize_compressed.encode(encoder)?;
        self.commit_finalize_compressed_and_undelegate
            .encode(encoder)?;
        encode_handler_slice(self.standalone_actions, self.indices_map, encoder)
    }
}

/// Encodes `AddActionCallbackArgs`
impl bincode::Encode for AddActionCallbackArgs<'_> {
    #[inline]
    fn encode<E: Encoder>(&self, encoder: &mut E) -> Result<(), EncodeError> {
        self.action_index.encode(encoder)?;
        self.destination_program.to_bytes().encode(encoder)?;
        self.discriminator.encode(encoder)?;
        self.payload.encode(encoder)?;
        self.compute_units.encode(encoder)?;
        self.accounts.encode(encoder)
    }
}

impl AddActionCallbackArgs<'_> {
    pub(super) fn encode_into_slice(&self, data_buf: &mut [u8]) -> Result<usize, ProgramError> {
        const OFFSET: usize = ADD_ACTION_CALLBACK_DISCRIMINANT.len();
        data_buf[..OFFSET].copy_from_slice(&ADD_ACTION_CALLBACK_DISCRIMINANT);
        let len =
            bincode::encode_into_slice(self, &mut data_buf[OFFSET..], bincode::config::legacy())
                .map_err(|_| ProgramError::InvalidInstructionData)?;
        Ok(OFFSET + len)
    }
}

// ---------------------------------------------------------------------------
// Helpers for standalone actions (no Intent wrapper exists for raw slices)
// ---------------------------------------------------------------------------

/// Encodes a `&[CallHandler]` as a bincode-legacy slice (u64 length + elements).
///
/// Each handler is encoded by constructing a [`BaseActionArgs`] locally (~80 bytes),
/// avoiding the 808-byte `NoVec<BaseActionArgs, MAX_ACTIONS_NUM>` allocation.
#[inline(never)]
fn encode_handler_slice<E: Encoder>(
    handlers: &[CallHandler<'_>],
    indices_map: &[&Address],
    encoder: &mut E,
) -> Result<(), EncodeError> {
    (handlers.len() as u64).encode(encoder)?;
    for handler in handlers {
        encode_handler(handler, indices_map, encoder)?;
    }
    Ok(())
}

/// Encodes a single [`CallHandler`] as a [`BaseActionArgs`] struct.
///
/// Constructs `BaseActionArgs` on the stack (~80 bytes) and delegates to its
/// derived [`bincode::Encode`] impl, so field order cannot silently diverge
/// from the canonical serialization type.
#[inline(never)]
#[allow(clippy::clone_on_copy)]
fn encode_handler<E: Encoder>(
    handler: &CallHandler<'_>,
    indices_map: &[&Address],
    encoder: &mut E,
) -> Result<(), EncodeError> {
    BaseActionArgs {
        args: handler.args.clone(),
        compute_units: handler.compute_units,
        destination_program: handler.destination_program.clone(),
        escrow_authority: get_index(indices_map, handler.escrow_authority.address())
            .ok_or(EncodeError::Other("escrow not in indices_map"))?,
        accounts: handler.accounts,
    }
    .encode(encoder)
}
