use pinocchio::Address;

use crate::vrf::consts::IDENTITY_SEED;

/// Derive the program identity PDA for `program_id` (seeds: `["identity"]`).
///
/// The VRF program requires the caller's program identity PDA to sign the
/// `RequestRandomness` instruction, proving the request originates from the
/// program that owns the callback. When requesting randomness from within a
/// program via CPI, sign with the seeds `[IDENTITY_SEED, &[bump]]`.
pub fn program_identity_pda(program_id: &Address) -> (Address, u8) {
    crate::pda::find_program_address(&[IDENTITY_SEED], program_id)
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
}
