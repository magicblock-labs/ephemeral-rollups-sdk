use core::mem::size_of;
use pinocchio::{error::ProgramError, Address};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Permission<'a> {
    pub discriminator: u8,
    pub bump: u8,
    pub permissioned_account: Address,
    pub members: Option<&'a [Member]>,
}

impl<'a> Permission<'a> {
    /// Calculate the exact size needed to serialize this Permission
    pub fn serialized_size(&self) -> usize {
        // discriminator (1) + bump (1) + address (32) = 34
        // + 1 byte option flag
        let mut size = 34 + 1;

        if let Some(members) = self.members {
            size += 4 + members.len() * MAX_MEMBER_SIZE;
        }

        size
    }

    /// Deserialize Permission from a data slice
    pub fn try_from_slice(data: &'a [u8]) -> Result<Self, ProgramError> {
        // minimum: 34 (base) + 1 (option)
        if data.len() < 35 {
            return Err(ProgramError::InvalidAccountData);
        }

        let discriminator = data[0];
        let bump = data[1];

        let permissioned_account =
            Address::try_from(&data[2..34]).map_err(|_| ProgramError::InvalidAccountData)?;

        let members_flag = data[34];

        let members = match members_flag {
            0 => None,
            1 => {
                // need at least count
                if data.len() < 39 {
                    return Err(ProgramError::InvalidAccountData);
                }

                let member_count_bytes: [u8; 4] = data[35..39]
                    .try_into()
                    .map_err(|_| ProgramError::InvalidAccountData)?;

                let member_count = u32::from_le_bytes(member_count_bytes) as usize;

                if member_count > MAX_MEMBERS_COUNT {
                    return Err(ProgramError::InvalidAccountData);
                }

                if member_count == 0 {
                    Some(&[])
                } else {
                    let members_start = 39;

                    let members_len = member_count
                        .checked_mul(MAX_MEMBER_SIZE)
                        .ok_or(ProgramError::InvalidAccountData)?;

                    let members_end = members_start
                        .checked_add(members_len)
                        .ok_or(ProgramError::InvalidAccountData)?;

                    if members_end > data.len() {
                        return Err(ProgramError::InvalidAccountData);
                    }

                    let members_data = &data[members_start..members_end];

                    let members_slice = unsafe {
                        core::slice::from_raw_parts(
                            members_data.as_ptr() as *const Member,
                            member_count,
                        )
                    };

                    Some(members_slice)
                }
            }
            _ => return Err(ProgramError::InvalidAccountData),
        };

        Ok(Permission {
            discriminator,
            bump,
            permissioned_account,
            members,
        })
    }

    /// Serialize Permission to a mutable byte slice
    pub fn try_to_slice<'b>(&self, data: &'b mut [u8]) -> Result<&'b [u8], ProgramError> {
        let required_size = self.serialized_size();

        if data.len() < required_size {
            return Err(ProgramError::AccountDataTooSmall);
        }

        // base fields
        data[0] = self.discriminator;
        data[1] = self.bump;
        data[2..34].copy_from_slice(self.permissioned_account.as_ref());

        let mut offset = 34;

        match self.members {
            None => {
                data[offset] = 0;
                offset += 1;
            }
            Some(members) => {
                data[offset] = 1;
                offset += 1;

                let member_count = members.len();
                data[offset..offset + 4]
                    .copy_from_slice(&(member_count as u32).to_le_bytes());
                offset += 4;

                for member in members {
                    data[offset] = member.flags.as_u8();
                    offset += 1;

                    data[offset..offset + 32]
                        .copy_from_slice(member.pubkey.as_ref());
                    offset += 32;
                }
            }
        }

        Ok(&data[..offset])
    }
}

#[repr(C)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Member {
    pub flags: MemberFlags,
    pub pubkey: Address,
}

pub const MAX_MEMBERS_COUNT: usize = 32;
pub const MAX_MEMBER_SIZE: usize = size_of::<u8>() + size_of::<Address>(); // flags + address = 33 bytes

const _: () = assert!(core::mem::size_of::<Member>() == MAX_MEMBER_SIZE);
pub const MAX_MEMBERS_ARGS_SIZE: usize = size_of::<u8>() // option flag
     + size_of::<u32>() // count
     + MAX_MEMBERS_COUNT * MAX_MEMBER_SIZE; // up to 32 members

pub struct MembersArgs<'a> {
    pub members: Option<&'a [Member]>,
}

impl<'a> MembersArgs<'a> {
    pub fn public() -> Self {
        MembersArgs { members: None }
    }

    pub fn private() -> Self {
        const EMPTY_MEMBERS: [Member; 0] = [];
        MembersArgs {
            members: Some(&EMPTY_MEMBERS),
        }
    }

    pub fn with_default_flags(
        addresses: &[Address],
        members_buf: &'a mut [Member],
    ) -> Result<Self, ProgramError> {
        if members_buf.len() < addresses.len() {
            return Err(ProgramError::InvalidArgument);
        }

        for (i, pubkey) in addresses.iter().enumerate() {
            members_buf[i] = Member {
                flags: MemberFlags::default(),
                #[allow(clippy::clone_on_copy)]
                pubkey: pubkey.clone(),
            };
        }

        Ok(MembersArgs {
            members: Some(&members_buf[..addresses.len()]),
        })
    }
}

impl MembersArgs<'_> {
    /// Calculate the exact size needed to serialize these args
    pub fn serialized_size(&self) -> usize {
        // 1 byte for option + 4 bytes for count (if Some) + member data
        let mut size = 1; // option byte

        if let Some(members) = self.members {
            size += 4 + members.len() * MAX_MEMBER_SIZE;
        }

        size
    }

    pub fn try_to_slice<'b>(&self, data: &'b mut [u8]) -> Result<&'b [u8], ProgramError> {
        if data.is_empty() {
            return Err(ProgramError::InvalidArgument);
        }

        match self.members {
            Some(members) => {
                // Write option byte: 1 for Some
                data[0] = 1;

                // Need at least 1 (option) + 4 (count) bytes
                if data.len() < 5 {
                    return Err(ProgramError::InvalidArgument);
                }

                let member_count = members.len();

                // Check size
                if 5 + member_count * MAX_MEMBER_SIZE > data.len() {
                    return Err(ProgramError::InvalidArgument);
                }

                // Serialize count at offset 1
                let count_bytes = (member_count as u32).to_le_bytes();
                data[1..5].copy_from_slice(&count_bytes);

                let mut offset = 5;

                // Serialize members
                for member in members {
                    // Flags (1 byte)
                    data[offset] = member.flags.as_u8();
                    offset += 1;

                    // Address (32 bytes)
                    data[offset..offset + 32].copy_from_slice(member.pubkey.as_ref());
                    offset += 32;
                }

                Ok(&data[..offset])
            }
            None => {
                // Write option byte: 0 for None
                data[0] = 0;
                Ok(&data[..1])
            }
        }
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
#[repr(transparent)]
pub struct MemberFlags(u8);

impl MemberFlags {
    pub const AUTHORITY: u8 = 1 << 0;
    pub const TX_LOGS: u8 = 1 << 1;
    pub const TX_BALANCES: u8 = 1 << 2;
    pub const TX_MESSAGE: u8 = 1 << 3;
    pub const ACCOUNT_SIGNATURES: u8 = 1 << 4;

    pub fn new() -> Self {
        MemberFlags(0)
    }

    pub fn has(&self, flag: u8) -> bool {
        self.0 & flag != 0
    }

    pub fn set(&mut self, flag: u8) {
        self.0 |= flag;
    }

    pub fn remove(&mut self, flag: u8) {
        self.0 &= !flag;
    }

    pub fn as_u8(&self) -> u8 {
        self.0
    }

    /// Build flags from a single-byte bitfield.
    pub fn from_acl_flag_byte(byte: u8) -> Self {
        MemberFlags(byte)
    }

    /// Convert flags into a single-byte bitfield.
    pub fn to_acl_flag_byte(&self) -> u8 {
        self.0
    }

    /// Build flags from individual boolean values.
    pub fn from_acl_flags(
        authority: bool,
        tx_logs: bool,
        tx_balances: bool,
        tx_message: bool,
        account_signatures: bool,
    ) -> Self {
        let mut flags = MemberFlags::new();
        if authority {
            flags.set(MemberFlags::AUTHORITY);
        }
        if tx_logs {
            flags.set(MemberFlags::TX_LOGS);
        }
        if tx_balances {
            flags.set(MemberFlags::TX_BALANCES);
        }
        if tx_message {
            flags.set(MemberFlags::TX_MESSAGE);
        }
        if account_signatures {
            flags.set(MemberFlags::ACCOUNT_SIGNATURES);
        }
        flags
    }
}
