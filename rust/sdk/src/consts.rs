pub use dlp_api::consts::*;
use magicblock_magic_program_api::{MAGIC_ACCOUNT_VAULT_PUBKEY, MAGIC_CONTEXT_PUBKEY};

/// The magic program ID.
pub const MAGIC_PROGRAM_ID: solana_program::pubkey::Pubkey =
    solana_program::pubkey::Pubkey::new_from_array(magicblock_magic_program_api::ID.to_bytes());

/// The magic context ID.
pub const MAGIC_CONTEXT_ID: solana_program::pubkey::Pubkey =
    solana_program::pubkey::Pubkey::new_from_array(MAGIC_CONTEXT_PUBKEY.to_bytes());

/// The vault ID that collects rent for Magic accounts.
pub const MAGIC_ACCOUNT_VAULT_ID: solana_program::pubkey::Pubkey =
    solana_program::pubkey::Pubkey::new_from_array(MAGIC_ACCOUNT_VAULT_PUBKEY.to_bytes());

/// Legacy name for [`MAGIC_ACCOUNT_VAULT_ID`].
#[deprecated(note = "use `MAGIC_ACCOUNT_VAULT_ID` instead")]
pub const EPHEMERAL_VAULT_ID: solana_program::pubkey::Pubkey = MAGIC_ACCOUNT_VAULT_ID;

/// The permission program ID.
pub const PERMISSION_PROGRAM_ID: solana_program::pubkey::Pubkey =
    solana_program::pubkey::Pubkey::from_str_const("ACLseoPoyC3cBqoUtkbjZ4aDrkurZW86v19pXz2XQnp1");

/// The ephemeral SPL token program ID.
pub const ESPL_TOKEN_PROGRAM_ID: solana_program::pubkey::Pubkey =
    solana_program::pubkey::Pubkey::from_str_const("SPLxh1LVZzEkX99H6rqYizhytLWPZVV296zyYDPagv2");

/// The Hydra program ID.
pub const HYDRA_PROGRAM_ID: solana_program::pubkey::Pubkey =
    solana_program::pubkey::Pubkey::from_str_const("Hydra17i1feui9deaxu6d1TzSQMRNHeBRkDR1Awy7zea");

/// The token program ID.
pub const TOKEN_PROGRAM_ID: solana_program::pubkey::Pubkey =
    solana_program::pubkey::Pubkey::from_str_const("TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA");

/// The associated token program ID.
pub const ASSOCIATED_TOKEN_PROGRAM_ID: solana_program::pubkey::Pubkey =
    solana_program::pubkey::Pubkey::from_str_const("ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL");
