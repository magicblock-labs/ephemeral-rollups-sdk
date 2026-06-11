use pinocchio::Address;

use crate::vrf::consts::{IDENTITY_SEED, VRF_PROGRAM_ID};

/// Derive the program identity PDA for `program_id` (seeds: `["identity"]`).
///
/// The VRF program requires the caller's program identity PDA to sign the
/// `RequestRandomness` instruction, proving the request originates from the
/// program that owns the callback. When requesting randomness from within a
/// program via CPI, sign with the seeds `[IDENTITY_SEED, &[bump]]`.
pub fn program_identity_pda(program_id: &Address) -> (Address, u8) {
    crate::pda::find_program_address(&[IDENTITY_SEED], program_id)
}

/// Derive the scoped VRF identity PDA for `callback_program_id`.
///
/// Scoped requests are fulfilled by the VRF program with this PDA as signer,
/// using seeds `["identity", callback_program_id]` under the VRF program id.
/// Callback handlers should validate this address instead of the deprecated
/// global [`crate::vrf::consts::VRF_PROGRAM_IDENTITY`] address.
pub fn scoped_vrf_identity(callback_program_id: &Address) -> (Address, u8) {
    crate::pda::find_program_address(
        &[IDENTITY_SEED, callback_program_id.as_ref()],
        &VRF_PROGRAM_ID,
    )
}

#[cfg(test)]
mod tests {
    use solana_program::pubkey::Pubkey;

    use super::*;

    #[test]
    fn program_identity_matches_canonical_derivation() {
        for salt in [0u8, 1, 42, 200] {
            let prog = [salt; 32];
            let (pda, bump) = program_identity_pda(&Address::new_from_array(prog));
            let (expected, expected_bump) =
                Pubkey::find_program_address(&[IDENTITY_SEED], &Pubkey::new_from_array(prog));
            assert_eq!(pda.as_ref(), expected.as_ref());
            assert_eq!(bump, expected_bump);
        }
    }

    #[test]
    fn scoped_vrf_identity_matches_canonical_derivation() {
        for salt in [0u8, 1, 42, 200] {
            let callback_program = Address::new_from_array([salt; 32]);
            let (pda, bump) = scoped_vrf_identity(&callback_program);
            let expected =
                ephemeral_vrf_sdk::consts::scoped_vrf_identity(&Pubkey::new_from_array([salt; 32]));
            let (expected_pda, expected_bump) = Pubkey::find_program_address(
                &[IDENTITY_SEED, Pubkey::new_from_array([salt; 32]).as_ref()],
                &Pubkey::new_from_array(VRF_PROGRAM_ID.to_bytes()),
            );

            assert_eq!(expected.as_ref(), expected_pda.as_ref());
            assert_eq!(pda.as_ref(), expected.as_ref());
            assert_eq!(bump, expected_bump);
        }
    }
}
