use pinocchio::Address;
use pinocchio_pubkey::pubkey;

/// The permission program ID.
pub const PERMISSION_PROGRAM_ID: Address =
    Address::new_from_array(pubkey!("ACLseoPoyC3cBqoUtkbjZ4aDrkurZW86v19pXz2XQnp1"));

/// The seed of a permission PDA.
pub const PERMISSION: &[u8] = b"permission:";

pub const CREATE_PERMISSION_DISCRIMINATOR: u64 = 0;
pub const UPDATE_PERMISSION_DISCRIMINATOR: u64 = 1;
pub const CLOSE_PERMISSION_DISCRIMINATOR: u64 = 2;
