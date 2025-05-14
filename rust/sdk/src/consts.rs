// NOTE: this should go into a core package that both the sdk + the program can depend on
use solana_program::pubkey;
use solana_program::pubkey::Pubkey;

pub use dlp::consts::*;

/// The magic program ID.
pub const MAGIC_PROGRAM_ID: Pubkey = pubkey!("Magic11111111111111111111111111111111111111");

/// The magic context ID.
pub const MAGIC_CONTEXT_ID: Pubkey = pubkey!("MagicContext1111111111111111111111111111111");
