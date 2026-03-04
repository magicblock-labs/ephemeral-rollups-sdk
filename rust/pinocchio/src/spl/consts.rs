use pinocchio::Address;
use pinocchio_pubkey::pubkey;

/// The ephemeral SPL token program ID.
pub const ESPL_TOKEN_PROGRAM_ID: Address =
    Address::new_from_array(pubkey!("SPLxh1LVZzEkX99H6rqYizhytLWPZVV296zyYDPagv2"));

/// The SPL token program ID.
pub const TOKEN_PROGRAM_ID: Address =
    Address::new_from_array(pubkey!("TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA"));

/// The associated token program ID.
pub const ASSOCIATED_TOKEN_PROGRAM_ID: Address =
    Address::new_from_array(pubkey!("ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL"));
