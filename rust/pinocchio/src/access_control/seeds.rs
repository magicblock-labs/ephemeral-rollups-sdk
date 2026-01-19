/// Seed definitions for permission program PDAs
use pinocchio::Address;

/// Represents all types of seeds used for permission PDAs
pub enum PermissionSeed<'a> {
    /// Permission PDA seed: b"permission:" + permissioned_account address
    Permission(&'a Address),
}

impl<'a> PermissionSeed<'a> {
    /// Fill a seed slice with the appropriate seed bytes
    ///
    /// Returns a slice of byte slices that can be used for PDA derivation
    pub fn fill_seed_slice<'b>(&'a self, out: &'b mut [&'a [u8]; 3]) -> &'b [&'a [u8]]
    where
        'b: 'a,
    {
        match self {
            PermissionSeed::Permission(pubkey) => {
                out[0] = b"permission:";
                out[1] = pubkey.as_ref();
                &out[..2]
            }
        }
    }
}
