// NOTE: this should go into a core package that both the sdk + the program can depend on
use solana_program::pubkey::Pubkey;

pub use dlp::consts::*;
use magicblock_core::magic_program::MAGIC_CONTEXT_PUBKEY;

/// The magic program ID.
pub const MAGIC_PROGRAM_ID: Pubkey = magicblock_core::magic_program::ID;

/// The magic context ID.
pub const MAGIC_CONTEXT_ID: Pubkey = MAGIC_CONTEXT_PUBKEY;
