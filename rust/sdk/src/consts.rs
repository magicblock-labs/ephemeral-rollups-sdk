use crate::solana_compat::solana::Pubkey;

pub use dlp_api::dlp::consts::*;
use magicblock_magic_program_api::{EPHEMERAL_VAULT_PUBKEY, MAGIC_CONTEXT_PUBKEY};

/// The magic program ID.
pub const MAGIC_PROGRAM_ID: Pubkey = magicblock_magic_program_api::ID;

/// The magic context ID.
pub const MAGIC_CONTEXT_ID: Pubkey = MAGIC_CONTEXT_PUBKEY;

/// The ephemeral vault ID (collects rent for ephemeral accounts).
pub const EPHEMERAL_VAULT_ID: Pubkey = EPHEMERAL_VAULT_PUBKEY;

/// The permission program ID.
pub const PERMISSION_PROGRAM_ID: Pubkey =
    Pubkey::from_str_const("ACLseoPoyC3cBqoUtkbjZ4aDrkurZW86v19pXz2XQnp1");

/// The ephemeral SPL token program ID.
pub const ESPL_TOKEN_PROGRAM_ID: Pubkey =
    Pubkey::from_str_const("SPLxh1LVZzEkX99H6rqYizhytLWPZVV296zyYDPagv2");

/// The token program ID.
pub const TOKEN_PROGRAM_ID: Pubkey =
    Pubkey::from_str_const("TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA");

/// The associated token program ID.
pub const ASSOCIATED_TOKEN_PROGRAM_ID: Pubkey =
    Pubkey::from_str_const("ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL");
