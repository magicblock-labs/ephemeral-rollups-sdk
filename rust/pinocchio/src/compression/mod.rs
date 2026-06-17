mod delegate;
mod initialize_record;

pub use delegate::*;
pub use initialize_record::*;

use pinocchio::error::ProgramError;

pub const INITIALIZE_COMPRESSED_RECORD_DISCRIMINATOR: [u8; 8] = 0_u64.to_le_bytes();
pub const DELEGATE_COMPRESSED_DISCRIMINATOR: [u8; 8] = 1_u64.to_le_bytes();
pub const COMMIT_STATE_DISCRIMINATOR: [u8; 8] = 2_u64.to_le_bytes();
pub const UNDELEGATE_COMPRESSED_DISCRIMINATOR: [u8; 8] = 3_u64.to_le_bytes();
pub const EXTERNAL_UNDELEGATE_DISCRIMINATOR: [u8; 8] =
    [0xD, 0x23, 0xB0, 0x7C, 0x70, 0x68, 0xFE, 0x73];
pub const MAX_ACCOUNT_DATA_SIZE: usize = 500;

/// Builds the borsh encoded PDA seeds.
pub fn build_pda_seeds<'a, const N: usize>(
    buf: &'a mut [u8; N],
    seeds: &[&'a [u8]],
) -> Result<&'a [u8], ProgramError> {
    let seeds_len_u32 = u32::try_from(seeds.len()).map_err(|_| ProgramError::InvalidArgument)?;
    let mut offset = 4usize;
    if offset > N {
        return Err(ProgramError::InvalidArgument);
    }
    buf[..4].copy_from_slice(&seeds_len_u32.to_le_bytes());

    for seed in seeds {
        let seed_len_u32 = u32::try_from(seed.len()).map_err(|_| ProgramError::InvalidArgument)?;
        let needed = offset
            .checked_add(4)
            .and_then(|v| v.checked_add(seed.len()))
            .ok_or(ProgramError::InvalidArgument)?;
        if needed > N {
            return Err(ProgramError::InvalidArgument);
        }
        buf[offset..offset + 4].copy_from_slice(&seed_len_u32.to_le_bytes());
        offset += 4;
        buf[offset..offset + seed.len()].copy_from_slice(seed);
        offset += seed.len();
    }
    Ok(&buf[..offset])
}

// ------------------------------------------------------------------------------------------------
// Reimplementations of the types from light-sdk for borsh compatibility
// ------------------------------------------------------------------------------------------------

/// Reimplements [ValidityProof] compatible with borsh 1.
/// It is a wrapper around a [CompressedProof] [Option] that is compatible with borsh 1.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct CdpValidityProof(pub Option<CdpCompressedProof>);

impl CdpValidityProof {
    pub const WIRE_LEN: usize = 129;

    pub fn parse(bytes: &[u8]) -> Result<(Self, usize), ProgramError> {
        if bytes.is_empty() {
            return Err(ProgramError::InvalidInstructionData);
        }
        match bytes[0] {
            0 => return Ok((Self(None), 1)),
            1 => {}
            _ => return Err(ProgramError::InvalidInstructionData),
        }
        if bytes.len() < Self::WIRE_LEN {
            return Err(ProgramError::InvalidInstructionData);
        }
        Ok((
            Self(Some(
                CdpCompressedProof::parse(&bytes[1..Self::WIRE_LEN])
                    .map_err(|_| ProgramError::InvalidInstructionData)?
                    .0,
            )),
            Self::WIRE_LEN,
        ))
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CdpCompressedProof {
    pub a: [u8; 32],
    pub b: [u8; 64],
    pub c: [u8; 32],
}

impl CdpCompressedProof {
    pub fn parse(bytes: &[u8]) -> Result<(Self, usize), ProgramError> {
        if bytes.len() < core::mem::size_of::<Self>() {
            return Err(ProgramError::InvalidInstructionData);
        }
        Ok((
            Self {
                a: bytes[..32]
                    .try_into()
                    .map_err(|_| ProgramError::InvalidInstructionData)?,
                b: bytes[32..96]
                    .try_into()
                    .map_err(|_| ProgramError::InvalidInstructionData)?,
                c: bytes[96..128]
                    .try_into()
                    .map_err(|_| ProgramError::InvalidInstructionData)?,
            },
            core::mem::size_of::<Self>(),
        ))
    }
}

/// Reimplements [CompressedAccountMeta] compatible with borsh 1.
/// It is a wrapper around a [CompressedAccountMeta] that is compatible with borsh 1.
#[derive(Clone, Copy, Debug, PartialEq, Default)]
pub struct CdpCompressedAccountMeta {
    pub tree_info: CdpPackedStateTreeInfo,
    pub address: [u8; 32],
    pub output_state_tree_index: u8,
}

impl CdpCompressedAccountMeta {
    pub const WIRE_LEN: usize = 42;

    pub fn parse(bytes: &[u8]) -> Result<(Self, usize), ProgramError> {
        if bytes.len() < Self::WIRE_LEN {
            return Err(ProgramError::InvalidInstructionData);
        }
        Ok((
            Self {
                tree_info: CdpPackedStateTreeInfo::parse(
                    &bytes[..CdpPackedStateTreeInfo::WIRE_LEN],
                )
                .map_err(|_| ProgramError::InvalidInstructionData)?
                .0,
                address: bytes[9..41]
                    .try_into()
                    .map_err(|_| ProgramError::InvalidInstructionData)?,
                output_state_tree_index: bytes[41],
            },
            Self::WIRE_LEN,
        ))
    }
}

/// Reimplements [PackedStateTreeInfo] compatible with borsh 1.
/// It is a wrapper around a [PackedStateTreeInfo] that is compatible with borsh 1.
#[derive(Clone, Copy, Debug, PartialEq, Default)]
pub struct CdpPackedStateTreeInfo {
    pub root_index: u16,
    pub prove_by_index: bool,
    pub merkle_tree_pubkey_index: u8,
    pub queue_pubkey_index: u8,
    pub leaf_index: u32,
}

impl CdpPackedStateTreeInfo {
    pub const WIRE_LEN: usize = 9;

    pub fn parse(bytes: &[u8]) -> Result<(Self, usize), ProgramError> {
        if bytes.len() < Self::WIRE_LEN {
            return Err(ProgramError::InvalidInstructionData);
        }
        Ok((
            Self {
                root_index: u16::from_le_bytes(
                    bytes[0..2]
                        .try_into()
                        .map_err(|_| ProgramError::InvalidInstructionData)?,
                ),
                prove_by_index: match bytes[0] {
                    0 => false,
                    1 => true,
                    _ => return Err(ProgramError::InvalidInstructionData),
                },
                merkle_tree_pubkey_index: bytes[3],
                queue_pubkey_index: bytes[4],
                leaf_index: u32::from_le_bytes(
                    bytes[5..9]
                        .try_into()
                        .map_err(|_| ProgramError::InvalidInstructionData)?,
                ),
            },
            Self::WIRE_LEN,
        ))
    }
}

/// Reimplements [PackedAddressTreeInfo] compatible with borsh 1.
/// It is a wrapper around a [PackedAddressTreeInfo] that is compatible with borsh 1.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct CdpPackedAddressTreeInfo {
    pub address_merkle_tree_pubkey_index: u8,
    pub address_queue_pubkey_index: u8,
    pub root_index: u16,
}

impl CdpPackedAddressTreeInfo {
    pub const WIRE_LEN: usize = 4;

    pub fn parse(bytes: &[u8]) -> Result<(Self, usize), ProgramError> {
        if bytes.len() < Self::WIRE_LEN {
            return Err(ProgramError::InvalidInstructionData);
        }
        Ok((
            Self {
                address_merkle_tree_pubkey_index: bytes[0],
                address_queue_pubkey_index: bytes[1],
                root_index: u16::from_le_bytes(
                    bytes[2..4]
                        .try_into()
                        .map_err(|_| ProgramError::InvalidInstructionData)?,
                ),
            },
            Self::WIRE_LEN,
        ))
    }
}
