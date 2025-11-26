#[allow(deprecated, clippy::all)]
mod generated;

use borsh::{BorshDeserialize, BorshSerialize};
use generated::accounts::{Group, Permission};
use generated::instructions::CreateGroupInstructionArgs;
pub use generated::programs::MAGICBLOCK_PERMISSION_PROGRAM_ID as ID;
pub use generated::*;

use crate::generated::instructions::{
    CreateGroupInstructionData, CreatePermissionInstructionData, UpdatePermissionInstructionData,
};

impl Group {
    pub const LEN: usize = 1 + 1 + 4 + 32 * 32;
    pub const DISCRIMINATOR: u8 = 1;
}

impl Permission {
    pub const DISCRIMINATOR: u8 = 0;
}

/// Helper trait to make the generated code compatible with borsh 1.5
trait BorshCompatibility
where
    Self: BorshDeserialize + BorshSerialize,
{
    fn try_to_vec(&self) -> Result<Vec<u8>, std::io::Error> {
        let mut data = Vec::new();
        self.serialize(&mut data)?;
        Ok(data)
    }
}

impl BorshCompatibility for Group {}
impl BorshCompatibility for Permission {}
impl BorshCompatibility for CreateGroupInstructionArgs {}
impl BorshCompatibility for CreateGroupInstructionData {}
impl BorshCompatibility for CreatePermissionInstructionData {}
impl BorshCompatibility for UpdatePermissionInstructionData {}
