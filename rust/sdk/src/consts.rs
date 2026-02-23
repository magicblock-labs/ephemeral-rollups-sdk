use crate::solana_compat::solana::Pubkey;

pub use dlp::consts::*;
use magicblock_magic_program_api::MAGIC_CONTEXT_PUBKEY;

/// The magic program ID.
pub const MAGIC_PROGRAM_ID: Pubkey =
    Pubkey::new_from_array(magicblock_magic_program_api::ID.to_bytes());

/// The magic context ID.
pub const MAGIC_CONTEXT_ID: Pubkey = Pubkey::new_from_array(MAGIC_CONTEXT_PUBKEY.to_bytes());

/// The permission program ID.
pub const PERMISSION_PROGRAM_ID: Pubkey =
    Pubkey::from_str_const("ACLseoPoyC3cBqoUtkbjZ4aDrkurZW86v19pXz2XQnp1");

/// The ephemeral SPL token program ID.
pub const ESPL_TOKEN_PROGRAM_ID: Pubkey =
    Pubkey::from_str_const("SPLxh1LVZzEkX99H6rqYizhytLWPZVV296zyYDPagv2");

/// The token program ID.
pub const TOKEN_PROGRAM_ID: Pubkey =
    Pubkey::from_str_const("TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA");
