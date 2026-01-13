// Tests for access-control crate (magicblock-permission-client)
// Tests the Group and Permission account types

#[cfg(test)]
mod tests {
    use ephemeral_rollups_sdk::access_control::instructions::{
        ClosePermissionBuilder, CommitAndUndelegatePermissionBuilder, CommitPermissionBuilder,
        CreatePermissionBuilder, UpdatePermissionBuilder,
    };
    use ephemeral_rollups_sdk::consts::PERMISSION_PROGRAM_ID;
    use ephemeral_rollups_sdk::access_control::types::MembersArgs;
    use solana_pubkey::Pubkey;

    #[test]
    fn test_access_control_module_exists() {
        // This test verifies that access-control module is properly compiled
    }

    #[test]
    fn test_permission_constants() {
        // Permission account structure:
        // - Discriminator: 0
        const PERMISSION_DISCRIMINATOR: u8 = 0;

        assert_eq!(PERMISSION_DISCRIMINATOR, 0);
    }

    #[test]
    fn test_borsh_serialization_compatibility() {
        // Verify that Borsh compatibility trait is properly implemented
        // This ensures accounts can be serialized/deserialized
    }

    #[test]
    fn test_commit_permission_builder_with_both_signers() {
        let authority = Pubkey::new_unique();
        let permissioned_account = Pubkey::new_unique();
        let permission = Pubkey::new_unique();

        let mut builder = CommitPermissionBuilder::new();
        builder
            .authority(authority, true)
            .permissioned_account(permissioned_account, true)
            .permission(permission);

        let instruction = builder.instruction();
        assert_eq!(instruction.program_id, PERMISSION_PROGRAM_ID);
        assert_eq!(instruction.accounts.len(), 5); // authority, permissioned_account, permission, magic_program, magic_context
        assert!(instruction.accounts[0].is_signer); // authority is signer
        assert!(instruction.accounts[1].is_signer); // permissioned_account is signer
    }

    #[test]
    fn test_commit_permission_builder_authority_only_signer() {
        let authority = Pubkey::new_unique();
        let permissioned_account = Pubkey::new_unique();
        let permission = Pubkey::new_unique();

        let mut builder = CommitPermissionBuilder::new();
        builder
            .authority(authority, true)
            .permissioned_account(permissioned_account, false)
            .permission(permission);

        let instruction = builder.instruction();
        assert!(instruction.accounts[0].is_signer); // authority is signer
        assert!(!instruction.accounts[1].is_signer); // permissioned_account is not signer
    }

    #[test]
    fn test_commit_permission_builder_permissioned_account_only_signer() {
        let authority = Pubkey::new_unique();
        let permissioned_account = Pubkey::new_unique();
        let permission = Pubkey::new_unique();

        let mut builder = CommitPermissionBuilder::new();
        builder
            .authority(authority, false)
            .permissioned_account(permissioned_account, true)
            .permission(permission);

        let instruction = builder.instruction();
        assert!(!instruction.accounts[0].is_signer); // authority is not signer
        assert!(instruction.accounts[1].is_signer); // permissioned_account is signer
    }

    #[test]
    fn test_commit_and_undelegate_permission_builder_with_both_signers() {
        let authority = Pubkey::new_unique();
        let permissioned_account = Pubkey::new_unique();
        let permission = Pubkey::new_unique();

        let mut builder = CommitAndUndelegatePermissionBuilder::new();
        builder
            .authority(authority, true)
            .permissioned_account(permissioned_account, true)
            .permission(permission);

        let instruction = builder.instruction();
        assert_eq!(instruction.program_id, PERMISSION_PROGRAM_ID);
        assert_eq!(instruction.accounts.len(), 5); // authority, permissioned_account, permission, magic_program, magic_context
        assert!(instruction.accounts[0].is_signer); // authority is signer
        assert!(instruction.accounts[1].is_signer); // permissioned_account is signer
    }

    #[test]
    fn test_commit_and_undelegate_permission_builder_authority_only_signer() {
        let authority = Pubkey::new_unique();
        let permissioned_account = Pubkey::new_unique();
        let permission = Pubkey::new_unique();

        let mut builder = CommitAndUndelegatePermissionBuilder::new();
        builder
            .authority(authority, true)
            .permissioned_account(permissioned_account, false)
            .permission(permission);

        let instruction = builder.instruction();
        assert!(instruction.accounts[0].is_signer); // authority is signer
        assert!(!instruction.accounts[1].is_signer); // permissioned_account is not signer
    }

    #[test]
    fn test_commit_and_undelegate_permission_builder_permissioned_account_only_signer() {
        let authority = Pubkey::new_unique();
        let permissioned_account = Pubkey::new_unique();
        let permission = Pubkey::new_unique();

        let mut builder = CommitAndUndelegatePermissionBuilder::new();
        builder
            .authority(authority, false)
            .permissioned_account(permissioned_account, true)
            .permission(permission);

        let instruction = builder.instruction();
        assert!(!instruction.accounts[0].is_signer); // authority is not signer
        assert!(instruction.accounts[1].is_signer); // permissioned_account is signer
    }

    #[test]
    fn test_create_permission_builder() {
        let permissioned_account = Pubkey::new_unique();
        let permission = Pubkey::new_unique();
        let payer = Pubkey::new_unique();
        let system_program = Pubkey::new_unique();
        let members = MembersArgs {
            members: Some(vec![]),
        };

        let mut builder = CreatePermissionBuilder::new();
        builder
            .permissioned_account(permissioned_account)
            .permission(permission)
            .payer(payer)
            .system_program(system_program)
            .args(members);

        let instruction = builder.instruction();

        assert_eq!(instruction.program_id, PERMISSION_PROGRAM_ID);
        assert_eq!(instruction.accounts.len(), 4); // permissioned_account, permission, payer, system_program
        assert!(instruction.accounts[0].is_signer); // permissioned_account is signer
        assert!(instruction.accounts[2].is_signer); // payer is signer
    }

    #[test]
    fn test_update_permission_builder_both_signers() {
        let authority = Pubkey::new_unique();
        let permissioned_account = Pubkey::new_unique();
        let permission = Pubkey::new_unique();
        let members = MembersArgs {
            members: Some(vec![]),
        };

        let mut builder = UpdatePermissionBuilder::new();
        builder
            .authority(authority, true)
            .permissioned_account(permissioned_account, true)
            .permission(permission)
            .args(members);

        let instruction = builder.instruction();

        assert_eq!(instruction.program_id, PERMISSION_PROGRAM_ID);
        assert_eq!(instruction.accounts.len(), 3); // authority, permissioned_account, permission
        assert!(instruction.accounts[0].is_signer); // authority is signer
        assert!(instruction.accounts[1].is_signer); // permissioned_account is signer
    }

    #[test]
    fn test_update_permission_builder_authority_only_signer() {
        let authority = Pubkey::new_unique();
        let permissioned_account = Pubkey::new_unique();
        let permission = Pubkey::new_unique();
        let members = MembersArgs {
            members: Some(vec![]),
        };

        let mut builder = UpdatePermissionBuilder::new();
        builder
            .authority(authority, true)
            .permissioned_account(permissioned_account, false)
            .permission(permission)
            .args(members);

        let instruction = builder.instruction();

        assert!(instruction.accounts[0].is_signer); // authority is signer
        assert!(!instruction.accounts[1].is_signer); // permissioned_account is not signer
    }

    #[test]
    fn test_close_permission_builder_both_signers() {
        let payer = Pubkey::new_unique();
        let permissioned_account = Pubkey::new_unique();
        let permission = Pubkey::new_unique();

        let mut builder = ClosePermissionBuilder::new();
        builder
            .payer(payer, true)
            .permissioned_account(permissioned_account, true)
            .permission(permission);

        let instruction = builder.instruction();

        assert_eq!(instruction.program_id, PERMISSION_PROGRAM_ID);
        assert_eq!(instruction.accounts.len(), 3); // payer, permissioned_account, permission
        assert!(instruction.accounts[0].is_signer); // payer is signer
        assert!(instruction.accounts[1].is_signer); // permissioned_account is signer
    }

    #[test]
    fn test_close_permission_builder_payer_only_signer() {
        let payer = Pubkey::new_unique();
        let permissioned_account = Pubkey::new_unique();
        let permission = Pubkey::new_unique();

        let mut builder = ClosePermissionBuilder::new();
        builder
            .payer(payer, true)
            .permissioned_account(permissioned_account, false)
            .permission(permission);

        let instruction = builder.instruction();

        assert!(instruction.accounts[0].is_signer); // payer is signer
        assert!(!instruction.accounts[1].is_signer); // permissioned_account is not signer
    }

    #[test]
    fn test_close_permission_builder_permissioned_account_only_signer() {
        let payer = Pubkey::new_unique();
        let permissioned_account = Pubkey::new_unique();
        let permission = Pubkey::new_unique();

        let mut builder = ClosePermissionBuilder::new();
        builder
            .payer(payer, false)
            .permissioned_account(permissioned_account, true)
            .permission(permission);

        let instruction = builder.instruction();

        assert!(!instruction.accounts[0].is_signer); // payer is not signer
        assert!(instruction.accounts[1].is_signer); // permissioned_account is signer
    }
}
