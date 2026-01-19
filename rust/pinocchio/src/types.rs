use core::mem::size_of;
use pinocchio::{address::MAX_SEEDS, address::MAX_SEED_LEN, error::ProgramError, Address};

pub const MAX_DELEGATE_ACCOUNT_ARGS_SIZE: usize = size_of::<u32>() // commit_frequency_ms
    + size_of::<u32>() // seeds length
    + MAX_SEEDS * (size_of::<u32>() + MAX_SEED_LEN) // seeds
    + 1 + size_of::<Address>(); // validator

#[derive(Debug)]
pub struct DelegateAccountArgs<'a> {
    pub commit_frequency_ms: u32,
    pub seeds: &'a [&'a [u8]],
    pub validator: Option<Address>,
}

impl Default for DelegateAccountArgs<'_> {
    fn default() -> Self {
        DelegateAccountArgs {
            commit_frequency_ms: u32::MAX,
            seeds: &[],
            validator: None,
        }
    }
}

impl DelegateAccountArgs<'_> {
    pub fn try_to_slice<'b>(&self, data: &'b mut [u8]) -> Result<&'b [u8], ProgramError> {
        if self.seeds.len() >= MAX_SEEDS {
            return Err(ProgramError::InvalidArgument);
        }

        for seed in self.seeds {
            if seed.len() > MAX_SEED_LEN {
                return Err(ProgramError::InvalidArgument);
            }
        }

        if data.len() != MAX_DELEGATE_ACCOUNT_ARGS_SIZE {
            return Err(ProgramError::InvalidArgument);
        }

        let mut offset = 0;

        // Serialize commit_frequency_ms (4 bytes)
        data[offset..offset + 4].copy_from_slice(&self.commit_frequency_ms.to_le_bytes());
        offset += 4;

        // Serialize seeds length (4 bytes)
        data[offset..offset + 4].copy_from_slice(&(self.seeds.len() as u32).to_le_bytes());
        offset += 4;

        // Serialize each seed
        for seed in self.seeds {
            data[offset..offset + 4].copy_from_slice(&(seed.len() as u32).to_le_bytes());
            offset += 4;
            data[offset..offset + seed.len()].copy_from_slice(seed);
            offset += seed.len();
        }

        match &self.validator {
            Some(pubkey) => {
                data[offset] = 1;
                offset += 1;
                data[offset..offset + 32].copy_from_slice(pubkey.as_ref());
                offset += 32;
            }
            None => {
                data[offset] = 0;
                offset += 1;
            }
        }

        Ok(&data[..offset])
    }
}

pub struct DelegateConfig {
    pub commit_frequency_ms: u32,
    pub validator: Option<Address>,
}

impl Default for DelegateConfig {
    fn default() -> Self {
        DelegateConfig {
            commit_frequency_ms: DelegateAccountArgs::default().commit_frequency_ms,
            validator: DelegateAccountArgs::default().validator,
        }
    }
}

/// PERMISSION & MEMBERS
pub struct Permission<'a> {
    pub discriminator: u8,
    pub bump: u8,
    pub permissioned_account: Address,
    pub members: Option<&'a [Member]>,
}

pub struct Member {
    pub flags: MemberFlags,
    pub pubkey: Address,
}

pub const MAX_MEMBERS_COUNT: usize = 512;
pub const MAX_MEMBER_SIZE: usize = size_of::<u8>() + size_of::<Address>(); // flags + address = 33 bytes
pub const MAX_MEMBERS_ARGS_SIZE: usize = size_of::<u32>() // count
    + MAX_MEMBERS_COUNT * MAX_MEMBER_SIZE; // up to 512 members

pub struct MembersArgs<'a> {
    pub members: Option<&'a [Member]>,
}

impl<'a> MembersArgs<'a> {
    pub fn public() -> Self {
        MembersArgs { members: None }
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
                pubkey: pubkey.clone(),
            };
        }

        Ok(MembersArgs {
            members: Some(&members_buf[..addresses.len()]),
        })
    }
}

impl MembersArgs<'_> {
    pub fn try_to_slice<'b>(&self, data: &'b mut [u8]) -> Result<&'b [u8], ProgramError> {
        if data.len() < 4 {
            return Err(ProgramError::InvalidArgument);
        }

        let member_count = self.members.map(|m| m.len()).unwrap_or(0);

        // Check size
        if 4 + member_count * MAX_MEMBER_SIZE > data.len() {
            return Err(ProgramError::InvalidArgument);
        }

        // Serialize count
        let count_bytes = (member_count as u32).to_le_bytes();
        data[0..4].copy_from_slice(&count_bytes);

        let mut offset = 4;

        // Serialize members
        if let Some(members) = self.members {
            for member in members {
                // Flags (1 byte)
                data[offset] = member.flags.as_u8();
                offset += 1;

                // Address (32 bytes)
                data[offset..offset + 32].copy_from_slice(member.pubkey.as_ref());
                offset += 32;
            }
        }

        Ok(&data[..offset])
    }
}

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
}

impl Default for MemberFlags {
    fn default() -> Self {
        let mut flags = MemberFlags(0);
        flags.set(MemberFlags::AUTHORITY);
        flags.set(MemberFlags::TX_LOGS);
        flags.set(MemberFlags::TX_BALANCES);
        flags.set(MemberFlags::TX_MESSAGE);
        flags.set(MemberFlags::ACCOUNT_SIGNATURES);
        flags
    }
}
