#[allow(deprecated, clippy::all)]
pub mod generated;

// Re-export the commonly used types
pub use generated::BorshCompatibility;
pub use generated::instructions::{
    CreateGroup, CreateGroupBuilder, CreateGroupCpi, CreateGroupCpiBuilder, CreateGroupCpiAccounts,
    CreateGroupInstructionArgs, CreateGroupInstructionData, CreatePermission, CreatePermissionBuilder,
    CreatePermissionCpi, CreatePermissionCpiBuilder, CreatePermissionCpiAccounts,
    CreatePermissionInstructionData, UpdatePermission, UpdatePermissionBuilder, UpdatePermissionCpi,
    UpdatePermissionCpiBuilder, UpdatePermissionCpiAccounts, UpdatePermissionInstructionData,
};
pub use generated::accounts::{Group, Permission};
pub use generated::errors;
pub use generated::programs::MAGICBLOCK_PERMISSION_PROGRAM_ID;

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
