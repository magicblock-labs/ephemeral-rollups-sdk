#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(not(feature = "std"))]
extern crate alloc;

pub mod consts;
pub mod instruction;
pub mod pda;
pub mod seeds;
pub mod types;
pub mod utils;

pinocchio_pubkey::declare_id!("DELeGGvXpWV2fqJUhqcF5ZSYMS4JTLjteaAMARRSaeSh");
