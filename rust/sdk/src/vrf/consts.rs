use crate::compat::Pubkey;

/// Verifiable Random Function program id
pub const VRF_PROGRAM_ID: Pubkey =
    Pubkey::from_str_const("Vrf1RNUjXmQGjmQrQLvJHs9SNkvDJEsRVFPkfSQUwGz");

/// The default queue for randomness requests
pub const DEFAULT_QUEUE: Pubkey =
    Pubkey::from_str_const("Cuj97ggrhhidhbu39TijNVqE74xvKJ69gDervRUXAxGh");

/// The default queue for ephemeral randomness requests
pub const DEFAULT_EPHEMERAL_QUEUE: Pubkey =
    Pubkey::from_str_const("5hBR571xnXppuCPveTrctfTU7tJLSN94nq7kv7FRK5Tc");

/// The default test queue for randomness requests. This is used in tests and local development.
pub const DEFAULT_EPHEMERAL_TEST_QUEUE: Pubkey =
    Pubkey::from_str_const("Sc9MJUngNbQXSXGP3F67KvKwVnhaYn6kcioxXNVowYT");
pub const DEFAULT_TEST_QUEUE: Pubkey =
    Pubkey::from_str_const("GKE6d7iv8kCBrsxr78W3xVdjGLLLJnxsGiuzrsZCGEvb");

/// Vrf program identity PDA
pub const VRF_PROGRAM_IDENTITY: Pubkey =
    Pubkey::from_str_const("9irBy75QS2BN81FUgXuHcjqceJJRuc9oDkAe8TKVvvAw");

/// Seed of the identity PDA
pub const IDENTITY: &[u8] = b"identity";
