//! Pinocchio SDK for the MagicBlock Verifiable Random Function (VRF) program.
//!
//! This is the `no_std`, allocation-free counterpart to the canonical
//! `ephemeral_vrf_sdk`, intended for on-chain pinocchio programs that request
//! randomness via CPI and consume the result inside a callback instruction.
//!
//! - [`consts`]: program ids, default queues and instruction discriminators.
//! - [`types`]: the [`RequestRandomness`](types::RequestRandomness) payload and
//!   its [`SerializableAccountMeta`](types::SerializableAccountMeta), with
//!   Borsh-compatible streaming serialization (no heap allocation).
//! - [`instruction`]: the [`RequestRandomnessCpi`](instruction::RequestRandomnessCpi)
//!   CPI helper.
//! - [`pda`]: program identity PDA derivation.
//! - [`rnd`]: helpers to derive typed random values from a 32-byte VRF seed.
//!
//! # Requesting randomness via CPI
//!
//! A program requests randomness by signing with its own program-identity PDA
//! (`["identity"]`). The VRF program later invokes the callback instruction
//! described by `callback_program_id` / `callback_discriminator`.
//!
//! ```ignore
//! use ephemeral_rollups_pinocchio::vrf::{
//!     consts::IDENTITY, RequestRandomness, RequestRandomnessCpi,
//! };
//! use pinocchio::cpi::{Seed, Signer};
//!
//! // `program_identity` is the PDA ["identity"] of THIS program; `identity_bump`
//! // is its bump. `oracle_queue` is e.g. `vrf::consts::DEFAULT_QUEUE`.
//! let request = RequestRandomness {
//!     caller_seed: [0u8; 32],
//!     callback_program_id: *crate_program_id,
//!     callback_discriminator: &[/* callback ix discriminator */],
//!     callback_accounts_metas: &[],
//!     callback_args: &[],
//! };
//! let cpi = RequestRandomnessCpi::new(
//!     payer,
//!     program_identity,
//!     oracle_queue,
//!     system_program,
//!     slot_hashes,
//!     request,
//! );
//!
//! let bump = [identity_bump];
//! let seeds = [Seed::from(IDENTITY), Seed::from(&bump)];
//! let signer = Signer::from(&seeds);
//!
//! let mut data = [0u8; 256]; // >= cpi.serialized_size()
//! cpi.invoke_signed(&mut data, &[signer])?;
//! ```

pub mod consts;
pub mod instruction;
pub mod pda;
pub mod rnd;
pub mod types;

pub use consts::*;
pub use instruction::*;
pub use pda::*;
pub use rnd::*;
pub use types::*;
