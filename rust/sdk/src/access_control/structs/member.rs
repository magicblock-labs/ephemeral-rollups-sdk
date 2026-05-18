#[cfg(feature = "anchor")]
use anchor_lang::prelude::*;
use bytemuck::{Pod, Zeroable};

use crate::solana_compat::solana::ProgramError;
#[cfg(not(feature = "anchor"))]
use borsh::{BorshDeserialize, BorshSerialize};

use crate::solana_compat::solana::Pubkey;

#[cfg_attr(feature = "anchor", derive(AnchorSerialize, AnchorDeserialize))]
#[cfg_attr(not(feature = "anchor"), derive(BorshSerialize, BorshDeserialize))]
#[repr(C)]
#[derive(Copy, Clone, Debug, Eq, PartialEq, Pod, Zeroable)]
pub struct Member {
    pub flags: u8,
    pub pubkey: Pubkey,
}

impl Member {
    pub const SIZE: usize = core::mem::size_of::<Self>();
}

#[cfg_attr(feature = "anchor", derive(AnchorSerialize, AnchorDeserialize))]
#[cfg_attr(not(feature = "anchor"), derive(BorshSerialize, BorshDeserialize))]
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
    pub fn is_authority(&self, user: &Pubkey) -> bool {
        self.flags & AUTHORITY_FLAG != 0 && &self.pubkey == user
    }

    pub fn can_see_tx_logs(&self, user: &Pubkey) -> bool {
        self.flags & TX_LOGS_FLAG != 0 && &self.pubkey == user
    }

    pub fn can_see_tx_balances(&self, user: &Pubkey) -> bool {
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

#[derive(Debug)]
pub struct EphemeralMembersArgs {
    pub is_private: bool,
    pub members: Vec<Member>,
}

impl EphemeralMembersArgs {
    pub fn required_bytes(members: usize) -> usize {
        1 + members * Member::SIZE
    }

    pub fn to_bytes(&self, bytes: &mut [u8]) -> std::result::Result<usize, ProgramError> {
        let members_bytes = self
            .members
            .len()
            .checked_mul(Member::SIZE)
            .ok_or(ProgramError::InvalidArgument)?;
        let required = 1usize
            .checked_add(members_bytes)
            .ok_or(ProgramError::InvalidArgument)?;
        if bytes.len() < required {
            return Err(ProgramError::InvalidArgument);
        }

        bytes[0] = if self.is_private { 1 } else { 0 };
        let mut offset = 1;
        for member in self.members.iter() {
            bytes[offset] = member.flags;
            offset += 1;
            bytes[offset..offset + 32].copy_from_slice(member.pubkey.as_ref());
            offset += 32;
        }

        Ok(required)
    }
}
