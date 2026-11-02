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
/// [`scoped_vrf_identity_const`] instead (the default).
pub const VRF_PROGRAM_IDENTITY: Pubkey = pubkey!("9irBy75QS2BN81FUgXuHcjqceJJRuc9oDkAe8TKVvvAw");

/// Seed of the identity PDA
pub const IDENTITY: &[u8] = b"identity";

/// Compile-time program identity PDA: `PDA([IDENTITY], program_id)`.
///
/// This is the readonly signer on a randomness request. Pass the consumer
/// program's `crate::ID` so the PDA is derived at compile time:
///
/// ```ignore
/// const PROGRAM_IDENTITY: Pubkey =
///     ephemeral_vrf_sdk::consts::program_identity_pubkey(&crate::ID);
/// let ix = ephemeral_vrf_sdk::instructions::create_request_randomness_ix_const(
///     params,
///     PROGRAM_IDENTITY,
/// );
/// ```
pub const fn program_identity_pda(program_id: &Pubkey) -> ([u8; 32], u8) {
    const_crypto::ed25519::derive_program_address(&[IDENTITY], program_id.as_array())
}

/// [`program_identity_pda`] as a [`Pubkey`].
pub const fn program_identity_pubkey(program_id: &Pubkey) -> Pubkey {
    Pubkey::new_from_array(program_identity_pda(program_id).0)
}

/// Compile-time scoped VRF identity PDA:
/// `PDA([IDENTITY, callback_program_id], vrf)`.
pub const fn scoped_vrf_identity_const(callback_program_id: &Pubkey) -> ([u8; 32], u8) {
    const_crypto::ed25519::derive_program_address(
        &[IDENTITY, callback_program_id.as_array()],
        VRF_PROGRAM_ID.as_array(),
    )
}

/// Scoped, per-callback-program VRF identity PDA: `PDA([IDENTITY, callback_program_id], vrf)`.
///
/// Bound to a specific callback program. Prefer [`scoped_vrf_identity_const`] with
/// `&crate::ID` in a `const` item to avoid `find_program_address`.
///
/// `#[account(address = scoped_vrf_identity(&crate::ID))] pub vrf_program_identity: Signer<'info>`.
/// The global [`VRF_PROGRAM_IDENTITY`] is deprecated.
#[deprecated(
    note = "Derives the scoped identity PDA at runtime via find_program_address. Use `scoped_vrf_identity_const(&crate::ID)` in a `const` item."
)]
pub fn scoped_vrf_identity(callback_program_id: &Pubkey) -> Pubkey {
    use crate::compat::{Compat, Modern};
    crate::compat::latest::Pubkey::find_program_address(
        &[IDENTITY, callback_program_id.modern().as_ref()],
        &VRF_PROGRAM_ID.modern(),
    )
    .0
    .compat()
}

#[cfg(test)]
#[allow(deprecated)]
mod tests {
    use crate::compat::{Compat, Modern};

    use super::*;

    #[test]
    fn program_identity_pda_matches_find_program_address() {
        for salt in [0u8, 1, 42, 200] {
            let program_id = Pubkey::new_from_array([salt; 32]);
            let (pda, bump) = program_identity_pda(&program_id);
            let (expected, expected_bump) = crate::compat::latest::Pubkey::find_program_address(
                &[IDENTITY],
                &program_id.modern(),
            );
            assert_eq!(pda, expected.to_bytes());
            assert_eq!(bump, expected_bump);
            assert_eq!(
                program_identity_pubkey(&program_id).modern(),
                expected.compat().modern()
            );
        }
    }

    #[test]
    fn program_identity_pda_is_usable_as_const() {
        const PROGRAM_ID: Pubkey = Pubkey::new_from_array([7u8; 32]);
        const PDA: ([u8; 32], u8) = program_identity_pda(&PROGRAM_ID);
        const KEY: Pubkey = program_identity_pubkey(&PROGRAM_ID);

        let (expected, expected_bump) =
            crate::compat::latest::Pubkey::find_program_address(&[IDENTITY], &PROGRAM_ID.modern());
        assert_eq!(PDA.0, expected.to_bytes());
        assert_eq!(PDA.1, expected_bump);
        assert_eq!(KEY.modern(), expected);
    }

    #[test]
    fn scoped_vrf_identity_const_matches_runtime() {
        for salt in [0u8, 1, 42, 200] {
            let callback_program = Pubkey::new_from_array([salt; 32]);
            let (pda, bump) = scoped_vrf_identity_const(&callback_program);
            let expected = scoped_vrf_identity(&callback_program);
            let (expected_pda, expected_bump) = crate::compat::latest::Pubkey::find_program_address(
                &[IDENTITY, callback_program.modern().as_ref()],
                &VRF_PROGRAM_ID.modern(),
            );
            assert_eq!(pda, expected.modern().to_bytes());
            assert_eq!(pda, expected_pda.to_bytes());
            assert_eq!(bump, expected_bump);
        }
    }
}
