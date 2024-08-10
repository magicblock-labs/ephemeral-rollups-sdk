use solana_program::pubkey::Pubkey;

pub use ephemeral_rollups_sdk_attribute_delegate::delegate;

pub mod anchor;
pub mod consts;
pub mod cpi;
pub mod delegate_args;
pub mod er;
pub mod pda;
pub mod types;
pub mod utils;

pub fn id() -> Pubkey {
    consts::DELEGATION_PROGRAM_ID
}
