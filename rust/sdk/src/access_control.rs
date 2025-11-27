#[cfg(feature = "access-control")]
pub use magicblock_permission_client::instructions::{
    CreateGroup, CreateGroupCpiBuilder, CreatePermission, CreatePermissionCpiBuilder,
    UpdatePermission, UpdatePermissionBuilder,
};

#[cfg(feature = "access-control")]
pub use magicblock_permission_client::accounts::{Group, Permission};

#[cfg(feature = "access-control")]
pub use magicblock_permission_client::ID as MAGICBLOCK_PERMISSION_PROGRAM_ID;
