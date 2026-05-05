pub mod close_ephemeral_permission;
pub mod close_permission;
pub mod commit_and_undelegate_permission;
pub mod commit_permission;
pub mod create_ephemeral_permission;
pub mod create_permission;
pub mod delegate_permission;
pub mod update_ephemeral_permission;
pub mod update_permission;

pub use close_ephemeral_permission::*;
pub use close_permission::*;
pub use commit_and_undelegate_permission::*;
pub use commit_permission::*;
pub use create_ephemeral_permission::*;
pub use create_permission::*;
pub use delegate_permission::*;
pub use update_ephemeral_permission::*;
pub use update_permission::*;

use crate::acl::MAX_MEMBER_SIZE;

pub const fn data_buffer_size(n: usize) -> usize {
    8 + 1 + n * MAX_MEMBER_SIZE
}
