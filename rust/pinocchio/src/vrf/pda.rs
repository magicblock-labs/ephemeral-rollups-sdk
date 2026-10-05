use pinocchio::Address;

use crate::vrf::consts::{IDENTITY_SEED, VRF_PROGRAM_ID};

/// Compile-time program identity PDA: `PDA(["identity"], program_id)`.
///
/// Prefer this with the consumer program's `crate::ID` so the PDA is derived at
/// compile time instead of via `find_program_address`:
///
/// ```ignore
/// const PROGRAM_IDENTITY: ([u8; 32], u8) =
///     ephemeral_rollups_pinocchio::vrf::program_identity_pda_const(&crate::ID);
/// ```
pub const fn program_identity_pda_const(program_id: &Address) -> ([u8; 32], u8) {
    const_crypto::ed25519::derive_program_address(&[IDENTITY_SEED], program_id.as_array())
}

/// Derive the program identity PDA for `program_id` (seeds: `["identity"]`).
///
/// The VRF program requires the caller's program identity PDA to sign the
/// `RequestRandomness` instruction, proving the request originates from the
/// program that owns the callback. When requesting randomness from within a
/// program via CPI, sign with the seeds `[IDENTITY_SEED, &[bump]]`.
#[deprecated(
    note = "Derives the identity PDA at runtime via find_program_address. Use `program_identity_pda_const(&crate::ID)` in a `const` item."
)]
pub fn program_identity_pda(program_id: &Address) -> (Address, u8) {
    crate::pda::find_program_address(&[IDENTITY_SEED], program_id)
}

/// Compile-time scoped VRF identity PDA:
/// `PDA(["identity", callback_program_id], vrf)`.
pub const fn scoped_vrf_identity_const(callback_program_id: &Address) -> ([u8; 32], u8) {
    const_crypto::ed25519::derive_program_address(
        &[IDENTITY_SEED, callback_program_id.as_array()],
        VRF_PROGRAM_ID.as_array(),
    )
}

/// Derive the scoped VRF identity PDA for `callback_program_id`.
///
/// Scoped requests are fulfilled by the VRF program with this PDA as signer,
/// using seeds `["identity", callback_program_id]` under the VRF program id.
/// Callback handlers should validate this address instead of the deprecated
/// global [`crate::vrf::consts::VRF_PROGRAM_IDENTITY`] address.
#[deprecated(
    note = "Derives the scoped identity PDA at runtime via find_program_address. Use `scoped_vrf_identity_const(&crate::ID)` in a `const` item."
)]
pub fn scoped_vrf_identity(callback_program_id: &Address) -> (Address, u8) {
    crate::pda::find_program_address(
        &[IDENTITY_SEED, callback_program_id.as_ref()],
        &VRF_PROGRAM_ID,
    )
}

#[cfg(test)]
#[allow(deprecated)]
mod tests {
    use solana_program::pubkey::Pubkey;

    use super::*;

    #[test]
    fn program_identity_matches_canonical_derivation() {
        for salt in [0u8, 1, 42, 200] {
            let prog = [salt; 32];
            #[allow(deprecated)]
            let (pda, bump) = program_identity_pda(&Address::new_from_array(prog));
            let (expected, expected_bump) =
                Pubkey::find_program_address(&[IDENTITY_SEED], &Pubkey::new_from_array(prog));
            let (const_pda, const_bump) =
                program_identity_pda_const(&Address::new_from_array(prog));
            assert_eq!(pda.as_ref(), expected.as_ref());
            assert_eq!(const_pda, expected.to_bytes());
            assert_eq!(
                const_pda,
                ephemeral_vrf_sdk::consts::program_identity_pda(&Pubkey::new_from_array(prog)).0
            );
            assert_eq!(bump, expected_bump);
            assert_eq!(const_bump, expected_bump);
        }
    }

    #[test]
    fn scoped_vrf_identity_matches_canonical_derivation() {
        for salt in [0u8, 1, 42, 200] {
            let callback_program = Address::new_from_array([salt; 32]);
            #[allow(deprecated)]
            let (pda, bump) = scoped_vrf_identity(&callback_program);
            let expected =
                ephemeral_vrf_sdk::consts::scoped_vrf_identity(&Pubkey::new_from_array([salt; 32]));
            let (expected_pda, expected_bump) = Pubkey::find_program_address(
                &[IDENTITY_SEED, Pubkey::new_from_array([salt; 32]).as_ref()],
                &Pubkey::new_from_array(VRF_PROGRAM_ID.to_bytes()),
            );
            let (const_pda, const_bump) = scoped_vrf_identity_const(&callback_program);

            assert_eq!(expected.as_ref(), expected_pda.as_ref());
            assert_eq!(pda.as_ref(), expected.as_ref());
            assert_eq!(const_pda, expected.as_ref());
            assert_eq!(
                const_pda,
                ephemeral_vrf_sdk::consts::scoped_vrf_identity_const(&Pubkey::new_from_array(
                    [salt; 32]
                ))
                .0
            );
            assert_eq!(bump, expected_bump);
            assert_eq!(const_bump, expected_bump);
        }
    }
}
