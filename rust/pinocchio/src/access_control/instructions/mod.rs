pub mod close_permission;
pub mod commit_and_undelegate_permission;
pub mod commit_permission;
/// Permission instruction builders for Pinocchio
///
/// Provides instruction factory functions for all permission operations
/// using stack-allocated buffers and custom serialization (no Vec, no Borsh)
pub mod create_permission;
pub mod delegate_permission;
pub mod undelegate_permission;
pub mod update_permission;

pub use close_permission::*;
pub use commit_and_undelegate_permission::*;
pub use commit_permission::*;
pub use create_permission::*;
pub use delegate_permission::*;
pub use undelegate_permission::*;
pub use update_permission::*;

/// Discriminators for each permission instruction
pub const CREATE_PERMISSION_DISCRIMINATOR: u64 = 0;
pub const UPDATE_PERMISSION_DISCRIMINATOR: u64 = 1;
pub const CLOSE_PERMISSION_DISCRIMINATOR: u64 = 2;
pub const DELEGATE_PERMISSION_DISCRIMINATOR: u64 = 3;
pub const COMMIT_PERMISSION_DISCRIMINATOR: u64 = 4;
pub const COMMIT_AND_UNDELEGATE_DISCRIMINATOR: u64 = 5;
pub const UNDELEGATE_PERMISSION_DISCRIMINATOR: u64 = 12048014319693667524;
