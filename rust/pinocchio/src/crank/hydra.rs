//! Hydra cranks: schedule instructions and verify they are executed by a crank.
//!
//! # Verifying the caller
//!
//! A scheduled instruction can check that it is executed by *your* crank by
//! reading the instructions sysvar. Pass the sysvar as a read-only account in
//! the scheduled instruction, then:
//!
//! ```ignore
//! use ephemeral_rollups_pinocchio::crank::hydra::{find_ephemeral_crank_pda, require_hydra_crank};
//!
//! // The crank your program created: `[b"crank", payer, seed]`.
//! let (expected_crank, _) = find_ephemeral_crank_pda(payer.address().as_array(), &seed);
//! // `0` = this is the first instruction scheduled in the crank.
//! require_hydra_crank(instructions_sysvar, &expected_crank, 0)?;
//! ```
//!
//! This replaces checking that the legacy crank signer PDA
//! (`crank_signer_pda`, seed `b"crank-executor"`) signed the instruction.
//! Crank signer verification is deprecated: it only works with the legacy
//! magic-program cranks, and Hydra rejects signers in scheduled instructions.

pub use hydra_api::{
    cpi::{base::pinocchio as base, ephemeral::pinocchio as ephemeral},
    instruction::CreateArgs,
    state::{find_base_crank_pda, find_ephemeral_base_crank_pda as find_ephemeral_crank_pda},
};

pub use super::verify::{hydra_crank_caller, require_hydra_crank};
