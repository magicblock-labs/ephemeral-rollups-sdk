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

use bincode::enc::Encoder;
use bincode::error::EncodeError;
use bincode::Encode;
use solana_address::Address;

use crate::intent_bundle::args::BaseActionArgs;
use crate::intent_bundle::types::{
    get_index, CallHandler, CommitAndUndelegateIntent, CommitIntent, MagicIntentBundle,
};

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
            standalone_actions: bundle.standalone_actions,
            indices_map,
        }
    }
}

impl bincode::Encode for MagicIntentBundleSerialize<'_, '_, '_> {
    #[inline(never)]
    fn encode<E: Encoder>(&self, encoder: &mut E) -> Result<(), EncodeError> {
        self.commit.encode(encoder)?;
        self.commit_and_undelegate.encode(encoder)?;
        self.commit_finalize.encode(encoder)?;
        self.commit_finalize_and_undelegate.encode(encoder)?;
        encode_handler_slice(self.standalone_actions, self.indices_map, encoder)
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
fn encode_handler<E: Encoder>(
    handler: &CallHandler<'_>,
    indices_map: &[&Address],
    encoder: &mut E,
) -> Result<(), EncodeError> {
    BaseActionArgs {
        args: handler.args.clone(),
        compute_units: handler.compute_units,
        destination_program: handler.destination_program,
        escrow_authority: get_index(indices_map, handler.escrow_authority.address())
            .ok_or(EncodeError::Other("escrow not in indices_map"))?,
        accounts: handler.accounts,
    }
    .encode(encoder)
}
