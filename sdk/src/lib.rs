use solana_program::pubkey::Pubkey;

pub use ephemeral_rollups_sdk_attribute_delegate::delegate;

pub mod consts;
pub mod delegate_args;
pub mod pda;
pub mod types;
pub mod utils;
pub mod cpi;
pub mod er;
pub mod anchor;

pub fn id() -> Pubkey {
    consts::DELEGATION_PROGRAM_ID
}