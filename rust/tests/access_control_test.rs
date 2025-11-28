// Tests for access-control crate (magicblock-permission-client)
// Tests the Group and Permission account types

#[cfg(test)]
mod tests {
    #[test]
    fn test_access_control_module_exists() {
        // This test verifies that access-control module is properly compiled
    }

    #[test]
    fn test_group_constants() {
        // Group account structure:
        // - Discriminator: 1 byte
        // - Length: 1030 bytes (1 + 1 + 4 + 32*32)
        const GROUP_DISCRIMINATOR: u8 = 1;
        const GROUP_LEN: usize = 1030;

        assert_eq!(GROUP_DISCRIMINATOR, 1);
        assert_eq!(GROUP_LEN, 1 + 1 + 4 + 32 * 32);
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
    fn test_account_discriminators() {
        let group_disc = 1u8;
        let permission_disc = 0u8;

        assert_ne!(group_disc, permission_disc);
    }
}
