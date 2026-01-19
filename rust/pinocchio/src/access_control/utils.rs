/// Utility functions for permission program integration
use crate::access_control::pinocchio::structs::{Member, Permission};
use pinocchio::Address;

/// Check if a user has authority flag in a permission account
pub fn check_authority(permission: &Permission, user: &Address) -> bool {
    if let Some(members) = &permission.members {
        members.iter().any(|m| m.is_authority(user))
    } else {
        false
    }
}

/// Check if a user can see transaction logs
pub fn check_tx_logs_access(permission: &Permission, user: &Address) -> bool {
    if let Some(members) = &permission.members {
        members.iter().any(|m| m.can_see_tx_logs(user))
    } else {
        false
    }
}

/// Check if a user can see transaction balances
pub fn check_tx_balances_access(permission: &Permission, user: &Address) -> bool {
    if let Some(members) = &permission.members {
        members.iter().any(|m| m.can_see_tx_balances(user))
    } else {
        false
    }
}

/// Check if a user has specific flags
pub fn check_flags(permission: &Permission, user: &Address, required_flags: u8) -> bool {
    if let Some(members) = &permission.members {
        members
            .iter()
            .any(|m| m.pubkey == *user && (m.flags & required_flags) == required_flags)
    } else {
        false
    }
}

/// Find member in permission list
pub fn find_member(permission: &Permission, user: &Address) -> Option<&Member> {
    permission
        .members
        .as_ref()
        .and_then(|members| members.iter().find(|m| m.pubkey == *user))
}

/// Get all members from permission
pub fn get_members(permission: &Permission) -> Option<&Vec<Member>> {
    permission.members.as_ref()
}

/// Count members in permission
pub fn member_count(permission: &Permission) -> usize {
    permission.members.as_ref().map(|m| m.len()).unwrap_or(0)
}

/// Check if permission has any members
pub fn has_members(permission: &Permission) -> bool {
    permission
        .members
        .as_ref()
        .map(|m| !m.is_empty())
        .unwrap_or(false)
}

/// Permission validation result
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PermissionCheckResult {
    /// User has required permissions
    Allowed,

    /// User does not have required permissions
    Denied,

    /// User not found in permission members
    UserNotFound,

    /// Permission account has no members
    NoMembers,
}

impl PermissionCheckResult {
    /// Check if permission check passed
    pub fn is_allowed(self) -> bool {
        matches!(self, PermissionCheckResult::Allowed)
    }
}

/// Comprehensive permission check
pub fn check_permission(
    permission: &Permission,
    user: &Address,
    required_flags: u8,
) -> PermissionCheckResult {
    if let Some(members) = &permission.members {
        if members.is_empty() {
            return PermissionCheckResult::NoMembers;
        }

        if let Some(member) = members.iter().find(|m| m.pubkey == *user) {
            if (member.flags & required_flags) == required_flags {
                PermissionCheckResult::Allowed
            } else {
                PermissionCheckResult::Denied
            }
        } else {
            PermissionCheckResult::UserNotFound
        }
    } else {
        PermissionCheckResult::NoMembers
    }
}
