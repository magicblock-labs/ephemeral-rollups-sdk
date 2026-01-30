#![no_std]

extern crate alloc;

pub mod acl;
pub mod consts;
pub mod instruction;
pub mod intent_bundle;
pub mod pda;
pub mod seeds;
pub mod types;
pub mod utils;

use pinocchio::Address;

pub use consts::DELEGATION_PROGRAM_ID as ID;

/// Returns a reference to the delegation program ID.
pub fn id() -> &'static Address {
    &ID
}
