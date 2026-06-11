use pinocchio::Address;
use pinocchio_pubkey::pubkey;

/// Verifiable Random Function program id.
pub const VRF_PROGRAM_ID: Address =
    Address::new_from_array(pubkey!("Vrf1RNUjXmQGjmQrQLvJHs9SNkvDJEsRVFPkfSQUwGz"));

/// The default queue for randomness requests.
pub const DEFAULT_QUEUE: Address =
    Address::new_from_array(pubkey!("Cuj97ggrhhidhbu39TijNVqE74xvKJ69gDervRUXAxGh"));

/// The default queue for ephemeral randomness requests.
pub const DEFAULT_EPHEMERAL_QUEUE: Address =
    Address::new_from_array(pubkey!("5hBR571xnXppuCPveTrctfTU7tJLSN94nq7kv7FRK5Tc"));

/// The default test queue for ephemeral randomness requests.
/// This is used in tests and local development.
pub const DEFAULT_EPHEMERAL_TEST_QUEUE: Address =
    Address::new_from_array(pubkey!("Sc9MJUngNbQXSXGP3F67KvKwVnhaYn6kcioxXNVowYT"));

/// The default test queue for randomness requests.
pub const DEFAULT_TEST_QUEUE: Address =
    Address::new_from_array(pubkey!("GKE6d7iv8kCBrsxr78W3xVdjGLLLJnxsGiuzrsZCGEvb"));

/// Vrf program identity PDA.
pub const VRF_PROGRAM_IDENTITY: Address =
    Address::new_from_array(pubkey!("9irBy75QS2BN81FUgXuHcjqceJJRuc9oDkAe8TKVvvAw"));

/// Seed of the identity PDA.
pub const IDENTITY_SEED: &[u8] = b"identity";

pub const REQUEST_RANDOMNESS_DISCRIMINATOR: u64 = 3;
pub const REQUEST_REGULAR_RANDOMNESS_DISCRIMINATOR: u64 = 8;

#[cfg(test)]
mod tests {
    use ephemeral_vrf_sdk::consts as sdk;

    use super::*;

    #[test]
    fn ids_match_sdk() {
        assert_eq!(VRF_PROGRAM_ID.as_ref(), sdk::VRF_PROGRAM_ID.as_ref());
        assert_eq!(DEFAULT_QUEUE.as_ref(), sdk::DEFAULT_QUEUE.as_ref());
        assert_eq!(
            DEFAULT_EPHEMERAL_QUEUE.as_ref(),
            sdk::DEFAULT_EPHEMERAL_QUEUE.as_ref()
        );
        assert_eq!(
            VRF_PROGRAM_IDENTITY.as_ref(),
            sdk::VRF_PROGRAM_IDENTITY.as_ref()
        );
        assert_eq!(
            DEFAULT_EPHEMERAL_TEST_QUEUE.as_ref(),
            sdk::DEFAULT_EPHEMERAL_TEST_QUEUE.as_ref()
        );
        assert_eq!(
            DEFAULT_TEST_QUEUE.as_ref(),
            sdk::DEFAULT_TEST_QUEUE.as_ref()
        );
        assert_eq!(IDENTITY_SEED, sdk::IDENTITY);
    }
}
