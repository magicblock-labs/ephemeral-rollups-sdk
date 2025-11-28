#[allow(deprecated, clippy::all)]
pub mod generated;

pub use generated::BorshCompatibility;

use generated::accounts::{Group, Permission};
use generated::instructions::CreateGroupInstructionArgs;
pub use generated::programs::MAGICBLOCK_PERMISSION_PROGRAM_ID;
pub use generated::*;

use generated::instructions::{
    CreateGroupInstructionData, CreatePermissionInstructionData, UpdatePermissionInstructionData,
};

impl Group {
    pub const LEN: usize = 1 + 1 + 4 + 32 * 32;
    pub const DISCRIMINATOR: u8 = 1;
}

impl Permission {
    pub const DISCRIMINATOR: u8 = 0;
}

impl BorshCompatibility for Group {}
impl BorshCompatibility for Permission {}
impl BorshCompatibility for CreateGroupInstructionArgs {}
impl BorshCompatibility for CreateGroupInstructionData {}
impl BorshCompatibility for CreatePermissionInstructionData {}
impl BorshCompatibility for UpdatePermissionInstructionData {}
