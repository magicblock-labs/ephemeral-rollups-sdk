use crate::solana_compat::solana::pubkey;
use crate::solana_compat::solana::Pubkey;

pub use dlp::consts::*;
use magicblock_magic_program_api::MAGIC_CONTEXT_PUBKEY;

/// The magic program ID.
pub const MAGIC_PROGRAM_ID: Pubkey =
    Pubkey::new_from_array(magicblock_magic_program_api::ID.to_bytes());

/// The magic context ID.
pub const MAGIC_CONTEXT_ID: Pubkey = Pubkey::new_from_array(MAGIC_CONTEXT_PUBKEY.to_bytes());

/// The delegation program ID.
pub const DELEGATION_PROGRAM_ID: Pubkey = pubkey!("DELeGGvXpWV2fqJUhqcF5ZSYMS4JTLjteaAMARRSaeSh");

/// The seed of the authority account PDA.
pub const DELEGATION_RECORD: &[u8] = b"delegation";

/// The account to store the delegated account seeds.
pub const DELEGATION_METADATA: &[u8] = b"delegation-metadata";

/// The seed of the buffer account PDA.
pub const BUFFER: &[u8] = b"buffer";

/// The seed of the committed state PDA.
pub const COMMIT_STATE: &[u8] = b"state-diff";

/// The seed of a commit state record PDA.
pub const COMMIT_RECORD: &[u8] = b"commit-state-record";

/// The discriminator for the external undelegate instruction.
pub const EXTERNAL_UNDELEGATE_DISCRIMINATOR: [u8; 8] = [196, 28, 41, 206, 48, 37, 51, 167];
