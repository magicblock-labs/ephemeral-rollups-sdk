#[allow(deprecated, clippy::all)]
pub mod generated;

// Re-export the commonly used types
pub use generated::accounts::{Group, Permission};
pub use generated::errors;
pub use generated::instructions::{
    CreateGroup, CreateGroupBuilder, CreateGroupCpi, CreateGroupCpiAccounts, CreateGroupCpiBuilder,
    CreateGroupInstructionArgs, CreateGroupInstructionData, CreatePermission,
    CreatePermissionBuilder, CreatePermissionCpi, CreatePermissionCpiAccounts,
    CreatePermissionCpiBuilder, CreatePermissionInstructionData, UpdatePermission,
    UpdatePermissionBuilder, UpdatePermissionCpi, UpdatePermissionCpiAccounts,
    UpdatePermissionCpiBuilder, UpdatePermissionInstructionData,
};
pub use generated::programs::MAGICBLOCK_PERMISSION_PROGRAM_ID;
pub use generated::BorshCompatibility;

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
