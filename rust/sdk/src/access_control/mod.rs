#[allow(deprecated, clippy::all)]
pub mod generated;

// ===== Re-exports =====
pub use generated::{
    accounts::{Group, Permission},
    errors,
    instructions::*,
    programs::MAGICBLOCK_PERMISSION_PROGRAM_ID,
    BorshCompatibility,
};

// ===== Account Constants =====
impl Group {
    pub const LEN: usize = 1 + 1 + 4 + 32 * 32;
    pub const DISCRIMINATOR: u8 = 1;
}

impl Permission {
    pub const DISCRIMINATOR: u8 = 0;
}

// ===== BorshCompatibility Implementations =====
macro_rules! impl_borsh {
    ($($t:ty),* $(,)?) => {
        $(impl BorshCompatibility for $t {})*
    };
}
impl_borsh!(
    Group,
    Permission,
    CreateGroupInstructionArgs,
    CreateGroupInstructionData,
    CreatePermissionInstructionData,
    UpdatePermissionInstructionData,
    ClosePermissionInstructionData,
    CommitAndUndelegatePermissionInstructionData,
    CommitPermissionInstructionData,
    DelegatePermissionInstructionData
);
