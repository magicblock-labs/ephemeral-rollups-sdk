use crate::compat::{pubkey, Pubkey};

/// Verifiable Random Function program id
pub const VRF_PROGRAM_ID: Pubkey = pubkey!("Vrf1RNUjXmQGjmQrQLvJHs9SNkvDJEsRVFPkfSQUwGz");

/// The default queue for randomness requests
pub const DEFAULT_QUEUE: Pubkey = pubkey!("Cuj97ggrhhidhbu39TijNVqE74xvKJ69gDervRUXAxGh");

/// The default queue for ephemeral randomness requests
pub const DEFAULT_EPHEMERAL_QUEUE: Pubkey = pubkey!("5hBR571xnXppuCPveTrctfTU7tJLSN94nq7kv7FRK5Tc");

/// The default test queue for randomness requests. This is used in tests and local development.
pub const DEFAULT_EPHEMERAL_TEST_QUEUE: Pubkey =
    pubkey!("Sc9MJUngNbQXSXGP3F67KvKwVnhaYn6kcioxXNVowYT");
pub const DEFAULT_TEST_QUEUE: Pubkey = pubkey!("GKE6d7iv8kCBrsxr78W3xVdjGLLLJnxsGiuzrsZCGEvb");

/// VRF program identity PDA (legacy, global). Deprecated: new integrations should validate
/// [`scoped_vrf_identity`] instead (the default).
pub const VRF_PROGRAM_IDENTITY: Pubkey = pubkey!("9irBy75QS2BN81FUgXuHcjqceJJRuc9oDkAe8TKVvvAw");

/// Seed of the identity PDA
pub const IDENTITY: &[u8] = b"identity";

/// Scoped, per-callback-program VRF identity PDA: `PDA([IDENTITY, callback_program_id], vrf)`.
///
/// Bound to a specific callback program. This is the default identity new consumers validate
/// in their callback's accounts, e.g.:
/// `#[account(address = scoped_vrf_identity(&crate::ID))] pub vrf_program_identity: Signer<'info>`.
/// The global [`VRF_PROGRAM_IDENTITY`] is deprecated.
pub fn scoped_vrf_identity(callback_program_id: &Pubkey) -> Pubkey {
    use crate::compat::{Compat, Modern};
    crate::compat::latest::Pubkey::find_program_address(
        &[IDENTITY, callback_program_id.modern().as_ref()],
        &VRF_PROGRAM_ID.modern(),
    )
    .0
    .compat()
}
