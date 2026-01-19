/// PDA derivation helpers for permission program
use crate::access_control::pinocchio::seeds::PermissionSeed;
use crate::consts::PERMISSION_PROGRAM_ID;
use pinocchio::Address;

/// Find the permission PDA for a given permissioned account
///
/// The permission PDA is derived from:
/// - Seed: b"permission:"
/// - Seed: permissioned_account pubkey
/// - Program ID: magicblock_permission_api
///
/// # Arguments
/// * `permissioned_account` - The account whose permissions are being managed
///
/// # Returns
/// A tuple of (PDA address, bump seed)
pub fn permission_pda_from_permissioned_account(permissioned_account: &Address) -> (Address, u8) {
    let mut buf: [&[u8]; 3] = [&[]; 3];
    let seed = PermissionSeed::Permission(permissioned_account);
    let seeds = seed.fill_seed_slice(&mut buf);
    Address::find_program_address(seeds, &PERMISSION_PROGRAM_ID)
}

/// Get the permission program ID
pub fn permission_program_id() -> Address {
    PERMISSION_PROGRAM_ID
}
