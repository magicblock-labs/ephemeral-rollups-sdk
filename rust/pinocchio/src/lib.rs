#![no_std]

extern crate alloc;

pub mod consts;
pub mod instruction;
pub mod pda;
pub mod seeds;
pub mod types;
pub mod utils;

use pinocchio::Address;
use pinocchio_pubkey::pubkey;

/// The delegation program ID as bytes.
pub const ID_BYTES: [u8; 32] = pubkey!("DELeGGvXpWV2fqJUhqcF5ZSYMS4JTLjteaAMARRSaeSh");

/// The delegation program ID.
pub const ID: Address = Address::new_from_array(ID_BYTES);

/// Returns a reference to the program ID.
pub fn id() -> &'static Address {
    &ID
}
