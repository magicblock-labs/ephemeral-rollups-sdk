/// Pinocchio-compatible type definitions
/// These are lightweight versions of structs used in pinocchio instructions
/// without any external serialization dependencies
use crate::consts::PERMISSION_PROGRAM_ID;
use pinocchio::Address;

pub const PERMISSION_SEED: &[u8] = b"permission:";

/// Member structure for permission management
/// Layout: flags (1 byte) + address (32 bytes) = 33 bytes
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Member {
    pub flags: u8,
    pub pubkey: Address,
}

/// Members arguments for instruction builders
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MembersArgs {
    pub members: Option<Vec<Member>>,
}

// Flags for Member
pub const AUTHORITY_FLAG: u8 = 1 << 0; // Member has authority privileges
pub const TX_LOGS_FLAG: u8 = 1 << 1; // Member can see transaction logs
pub const TX_BALANCES_FLAG: u8 = 1 << 2; // Member can see transaction balances
pub const TX_MESSAGE_FLAG: u8 = 1 << 3; // Member can see transaction messages
pub const ACCOUNT_SIGNATURES_FLAG: u8 = 1 << 4; // Member can see account signatures

impl Member {
    pub fn is_authority(&self, user: &Address) -> bool {
        self.flags & AUTHORITY_FLAG != 0 && &self.pubkey == user
    }

    pub fn can_see_tx_logs(&self, user: &Address) -> bool {
        self.flags & TX_LOGS_FLAG != 0 && &self.pubkey == user
    }

    pub fn can_see_tx_balances(&self, user: &Address) -> bool {
        self.flags & TX_BALANCES_FLAG != 0 && &self.pubkey == user
    }

    // Set multiple flags at once
    pub fn set_flags(&mut self, flags: u8) {
        self.flags |= flags;
    }

    // Remove multiple flags
    pub fn remove_flags(&mut self, flags: u8) {
        self.flags &= !flags;
    }
}

/// Permission structure for managing access control
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Permission {
    pub discriminator: u8,
    pub bump: u8,
    pub permissioned_account: Address,
    pub members: Option<Vec<Member>>,
}

impl Permission {
    /// Prefix values used to generate a PDA for this account.
    ///
    /// Values are positional and appear in the following order:
    ///
    ///   0. `PERMISSION_SEED`
    ///   1. permissioned_account (`Pubkey`)
    pub const PREFIX: &'static [u8] = PERMISSION_SEED;

    pub fn find_pda(permissioned_account: &Address) -> (Address, u8) {
        Address::find_program_address(
            &[PERMISSION_SEED, permissioned_account.as_ref()],
            &PERMISSION_PROGRAM_ID,
        )
    }
}
