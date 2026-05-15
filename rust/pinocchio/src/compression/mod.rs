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

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct CdpValidityProof(pub Option<[u8; 128]>);

impl CdpValidityProof {
    pub fn parse(bytes: &[u8]) -> Result<(Self, usize), ProgramError> {
        if bytes.is_empty() {
            return Err(ProgramError::InvalidInstructionData);
        }
        if bytes[0] == 0 {
            return Ok((Self(None), 1));
        }
        if bytes.len() < 129 {
            return Err(ProgramError::InvalidInstructionData);
        }
        Ok((
            Self(Some(
                bytes[1..129]
                    .try_into()
                    .map_err(|_| ProgramError::InvalidInstructionData)?,
            )),
            129,
        ))
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CdpCompressedAccountMeta(pub [u8; 42]);

impl CdpCompressedAccountMeta {
    pub fn parse(bytes: &[u8]) -> Result<(Self, usize), ProgramError> {
        if bytes.len() < core::mem::size_of::<Self>() {
            return Err(ProgramError::InvalidInstructionData);
        }
        Ok((
            Self(
                bytes[..42]
                    .try_into()
                    .map_err(|_| ProgramError::InvalidInstructionData)?,
            ),
            core::mem::size_of::<Self>(),
        ))
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct CdpPackedAddressTreeInfo(pub [u8; 4]);

impl CdpPackedAddressTreeInfo {
    pub fn parse(bytes: &[u8]) -> Result<(Self, usize), ProgramError> {
        if bytes.len() < core::mem::size_of::<Self>() {
            return Err(ProgramError::InvalidInstructionData);
        }
        Ok((
            Self(
                bytes[..4]
                    .try_into()
                    .map_err(|_| ProgramError::InvalidInstructionData)?,
            ),
            4,
        ))
    }
}

pub fn build_pda_seeds<'a, const N: usize>(buf: &'a mut [u8; N], seeds: &[&'a [u8]]) -> &'a [u8] {
    let mut offset = 4;
    buf[0..offset].copy_from_slice(&(seeds.len() as u32).to_le_bytes());
    for seed in seeds {
        buf[offset..offset + 4].copy_from_slice(&(seed.len() as u32).to_le_bytes());
        offset += 4;
        buf[offset..offset + seed.len()].copy_from_slice(seed);
        offset += seed.len();
    }
    &buf[..offset]
}
