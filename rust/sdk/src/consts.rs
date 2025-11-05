use crate::solana_compat::solana::Pubkey;

pub use dlp::consts::*;
use magicblock_magic_program_api::MAGIC_CONTEXT_PUBKEY;

/// The magic program ID.
pub const MAGIC_PROGRAM_ID: Pubkey =
    Pubkey::new_from_array(magicblock_magic_program_api::ID.to_bytes());

/// The magic context ID.
pub const MAGIC_CONTEXT_ID: Pubkey = Pubkey::new_from_array(MAGIC_CONTEXT_PUBKEY.to_bytes());
