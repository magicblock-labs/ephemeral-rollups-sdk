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

#[cfg(test)]
mod tests {
    use pinocchio::Address;

    use crate::acl::{EphemeralMembersArgs, Member, MemberFlags};

    use super::*;

    #[test]
    fn test_data_buffer_size() {
        // Substract discriminator
        let mut bytes = [0; data_buffer_size(0) - 8];
        let actual_len = EphemeralMembersArgs {
            is_private: false,
            members: &[],
        }
        .to_bytes(&mut bytes)
        .unwrap();
        assert_eq!(bytes.len(), actual_len);

        let mut bytes = [0; data_buffer_size(1) - 8];
        let actual_len = EphemeralMembersArgs {
            is_private: false,
            members: &[Member {
                flags: MemberFlags::default(),
                pubkey: Address::new_from_array([0; 32]),
            }],
        }
        .to_bytes(&mut bytes)
        .unwrap();
        assert_eq!(bytes.len(), actual_len);

        let mut bytes = [0; data_buffer_size(100) - 8];
        let actual_len = EphemeralMembersArgs {
            is_private: false,
            members: &core::array::from_fn::<_, 100, _>(|_| Member {
                flags: MemberFlags::default(),
                pubkey: Address::new_from_array([0; 32]),
            }),
        }
        .to_bytes(&mut bytes)
        .unwrap();
        assert_eq!(bytes.len(), actual_len);
    }
}
