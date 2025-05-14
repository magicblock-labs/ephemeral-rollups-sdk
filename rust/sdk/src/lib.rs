pub use dlp::pda;
use solana_program::pubkey::Pubkey;

#[cfg(feature = "anchor")]
pub mod anchor;
pub mod consts;
pub mod cpi;
pub mod delegate_args;
pub mod ephem;
pub mod types;
pub mod utils;

pub const fn id() -> Pubkey {
    consts::DELEGATION_PROGRAM_ID
}
