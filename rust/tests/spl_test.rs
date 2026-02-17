// Tests for spl crate
// Tests the SPL vault and token account instructions

#[cfg(test)]
mod tests {
    use ephemeral_rollups_sdk::{
        consts::{
            ESPL_TOKEN_PROGRAM_ID, MAGIC_CONTEXT_ID, MAGIC_PROGRAM_ID, PERMISSION_PROGRAM_ID,
            TOKEN_PROGRAM_ID,
        },
        spl::instructions::*,
    };
    use solana_pubkey::Pubkey;
    use solana_system_interface::program as system_program;

    #[test]
    fn test_spl_module_exists() {
        // This test verifies that spl module is properly compiled
    }

    #[test]
    fn test_initialize_global_vault() {
        let payer = Pubkey::new_unique();
        let vault = Pubkey::new_unique();
        let mint = Pubkey::new_unique();
        let vault_bump = 255u8;

        let instruction = initialize_global_vault(payer, vault, mint, vault_bump);

        assert_eq!(instruction.program_id, ESPL_TOKEN_PROGRAM_ID);
        assert_eq!(instruction.accounts.len(), 4);
        // vault (writable, not signer)
        assert_eq!(instruction.accounts[0].pubkey, vault);
        assert!(instruction.accounts[0].is_writable);
        assert!(!instruction.accounts[0].is_signer);
        // payer (writable, not signer)
        assert_eq!(instruction.accounts[1].pubkey, payer);
        assert!(instruction.accounts[1].is_writable);
        assert!(!instruction.accounts[1].is_signer);
        // mint (readonly)
        assert_eq!(instruction.accounts[2].pubkey, mint);
        assert!(!instruction.accounts[2].is_writable);
        assert!(!instruction.accounts[2].is_signer);
        // system_program (readonly)
        assert!(!instruction.accounts[3].is_writable);
        assert!(!instruction.accounts[3].is_signer);

        assert_eq!(
            instruction.data[0],
            EphemeralSplDiscriminator::InitializeGlobalVault as u8
        );
        assert_eq!(instruction.data[1], vault_bump);
    }

    #[test]
    fn test_initialize_ephemeral_ata() {
        let payer = Pubkey::new_unique();
        let eata = Pubkey::new_unique();
        let user = Pubkey::new_unique();
        let mint = Pubkey::new_unique();
        let eata_bump = 255u8;

        let instruction = initialize_ephemeral_ata(payer, eata, user, mint, eata_bump);

        assert_eq!(instruction.program_id, ESPL_TOKEN_PROGRAM_ID);
        assert_eq!(instruction.accounts.len(), 5);
        // eata (writable, not signer)
        assert_eq!(instruction.accounts[0].pubkey, eata);
        assert!(instruction.accounts[0].is_writable);
        assert!(!instruction.accounts[0].is_signer);
        // payer (writable, not signer)
        assert_eq!(instruction.accounts[1].pubkey, payer);
        assert!(instruction.accounts[1].is_writable);
        assert!(!instruction.accounts[1].is_signer);
        // user (readonly)
        assert_eq!(instruction.accounts[2].pubkey, user);
        assert!(!instruction.accounts[2].is_writable);
        assert!(!instruction.accounts[2].is_signer);
        // mint (readonly)
        assert_eq!(instruction.accounts[3].pubkey, mint);
        assert!(!instruction.accounts[3].is_writable);
        assert!(!instruction.accounts[3].is_signer);
        // system_program (readonly)
        assert!(!instruction.accounts[4].is_writable);
        assert!(!instruction.accounts[4].is_signer);

        assert_eq!(
            instruction.data[0],
            EphemeralSplDiscriminator::InitializeEphemeralAta as u8
        );
        assert_eq!(instruction.data[1], eata_bump);
    }

    #[test]
    fn test_deposit_spl_tokens() {
        let authority = Pubkey::new_unique();
        let eata = Pubkey::new_unique();
        let vault = Pubkey::new_unique();
        let mint = Pubkey::new_unique();
        let user_source_token_acc = Pubkey::new_unique();
        let vault_token_acc = Pubkey::new_unique();
        let amount = 1000u64;

        let instruction = deposit_spl_tokens(
            authority,
            eata,
            vault,
            mint,
            user_source_token_acc,
            vault_token_acc,
            amount,
        );

        assert_eq!(instruction.program_id, ESPL_TOKEN_PROGRAM_ID);
        assert_eq!(instruction.accounts.len(), 7);
        // eata (writable, not signer)
        assert_eq!(instruction.accounts[0].pubkey, eata);
        assert!(instruction.accounts[0].is_writable);
        assert!(!instruction.accounts[0].is_signer);
        // vault (readonly)
        assert_eq!(instruction.accounts[1].pubkey, vault);
        assert!(!instruction.accounts[1].is_writable);
        // mint (readonly)
        assert_eq!(instruction.accounts[2].pubkey, mint);
        assert!(!instruction.accounts[2].is_writable);
        // user_source_token_acc (writable)
        assert_eq!(instruction.accounts[3].pubkey, user_source_token_acc);
        assert!(instruction.accounts[3].is_writable);
        // vault_token_acc (writable)
        assert_eq!(instruction.accounts[4].pubkey, vault_token_acc);
        assert!(instruction.accounts[4].is_writable);
        // authority (readonly, signer)
        assert_eq!(instruction.accounts[5].pubkey, authority);
        assert!(!instruction.accounts[5].is_writable);
        assert!(instruction.accounts[5].is_signer);
        // TOKEN_PROGRAM_ID (readonly)
        assert_eq!(instruction.accounts[6].pubkey, TOKEN_PROGRAM_ID);
        assert!(!instruction.accounts[6].is_writable);

        assert_eq!(
            instruction.data[0],
            EphemeralSplDiscriminator::DepositSplTokens as u8
        );
        assert_eq!(
            u64::from_le_bytes(instruction.data[1..9].try_into().unwrap()),
            amount
        );
    }

    #[test]
    fn test_withdraw_spl_tokens() {
        let payer = Pubkey::new_unique();
        let eata = Pubkey::new_unique();
        let vault = Pubkey::new_unique();
        let mint = Pubkey::new_unique();
        let vault_ata = Pubkey::new_unique();
        let user_ata = Pubkey::new_unique();
        let eata_bump = 255u8;
        let amount = 1000u64;

        let instruction = withdraw_spl_tokens(
            payer, eata, vault, mint, vault_ata, user_ata, eata_bump, amount,
        );

        assert_eq!(instruction.program_id, ESPL_TOKEN_PROGRAM_ID);
        assert_eq!(instruction.accounts.len(), 7);
        // eata (writable)
        assert_eq!(instruction.accounts[0].pubkey, eata);
        assert!(instruction.accounts[0].is_writable);
        // vault (readonly)
        assert_eq!(instruction.accounts[1].pubkey, vault);
        assert!(!instruction.accounts[1].is_writable);
        // mint (readonly)
        assert_eq!(instruction.accounts[2].pubkey, mint);
        assert!(!instruction.accounts[2].is_writable);
        // vault_ata (writable)
        assert_eq!(instruction.accounts[3].pubkey, vault_ata);
        assert!(instruction.accounts[3].is_writable);
        // user_ata (writable)
        assert_eq!(instruction.accounts[4].pubkey, user_ata);
        assert!(instruction.accounts[4].is_writable);
        // payer (readonly, signer)
        assert_eq!(instruction.accounts[5].pubkey, payer);
        assert!(!instruction.accounts[5].is_writable);
        assert!(instruction.accounts[5].is_signer);
        // TOKEN_PROGRAM_ID (readonly)
        assert_eq!(instruction.accounts[6].pubkey, TOKEN_PROGRAM_ID);
        assert!(!instruction.accounts[6].is_writable);

        assert_eq!(
            instruction.data[0],
            EphemeralSplDiscriminator::WithdrawSplTokens as u8
        );
        assert_eq!(
            u64::from_le_bytes(instruction.data[1..9].try_into().unwrap()),
            amount
        );
        assert_eq!(instruction.data[9], eata_bump);
    }

    #[test]
    fn test_delegate_ephemeral_ata() {
        let payer = Pubkey::new_unique();
        let eata = Pubkey::new_unique();
        let delegation_buffer = Pubkey::new_unique();
        let delegation_record = Pubkey::new_unique();
        let delegation_metadata = Pubkey::new_unique();
        let eata_bump = 255u8;

        let instruction = delegate_ephemeral_ata(
            payer,
            eata,
            delegation_buffer,
            delegation_record,
            delegation_metadata,
            eata_bump,
            None,
        );

        assert_eq!(instruction.program_id, ESPL_TOKEN_PROGRAM_ID);
        assert_eq!(instruction.accounts.len(), 8);
        // payer (writable, signer)
        assert_eq!(instruction.accounts[0].pubkey, payer);
        assert!(instruction.accounts[0].is_writable);
        assert!(instruction.accounts[0].is_signer);
        // eata (writable)
        assert_eq!(instruction.accounts[1].pubkey, eata);
        assert!(instruction.accounts[1].is_writable);
        // ESPL_TOKEN_PROGRAM_ID (readonly)
        assert_eq!(instruction.accounts[2].pubkey, ESPL_TOKEN_PROGRAM_ID);
        assert!(!instruction.accounts[2].is_writable);
        // delegation_buffer (writable)
        assert_eq!(instruction.accounts[3].pubkey, delegation_buffer);
        assert!(instruction.accounts[3].is_writable);
        // delegation_record (writable)
        assert_eq!(instruction.accounts[4].pubkey, delegation_record);
        assert!(instruction.accounts[4].is_writable);
        // delegation_metadata (writable)
        assert_eq!(instruction.accounts[5].pubkey, delegation_metadata);
        assert!(instruction.accounts[5].is_writable);

        assert_eq!(
            instruction.data[0],
            EphemeralSplDiscriminator::DelegateEphemeralAta as u8
        );
        assert_eq!(instruction.data[1], eata_bump);
    }

    #[test]
    fn test_delegate_ephemeral_ata_some_validator() {
        let payer = Pubkey::new_unique();
        let eata = Pubkey::new_unique();
        let delegation_buffer = Pubkey::new_unique();
        let delegation_record = Pubkey::new_unique();
        let delegation_metadata = Pubkey::new_unique();
        let eata_bump = 255u8;
        let validator = Pubkey::new_unique();

        let instruction = delegate_ephemeral_ata(
            payer,
            eata,
            delegation_buffer,
            delegation_record,
            delegation_metadata,
            eata_bump,
            Some(validator),
        );

        assert_eq!(
            instruction.data[0],
            EphemeralSplDiscriminator::DelegateEphemeralAta as u8
        );
        assert_eq!(instruction.data[1], eata_bump);
        assert_eq!(instruction.data[2..34], validator.to_bytes());
    }

    #[test]
    fn test_undelegate_ephemeral_ata() {
        let payer = Pubkey::new_unique();
        let user_ata = Pubkey::new_unique();
        let eata = Pubkey::new_unique();

        let instruction = undelegate_ephemeral_ata(payer, user_ata, eata);

        assert_eq!(instruction.program_id, ESPL_TOKEN_PROGRAM_ID);
        assert_eq!(instruction.accounts.len(), 5);
        // payer (readonly, signer)
        assert_eq!(instruction.accounts[0].pubkey, payer);
        assert!(!instruction.accounts[0].is_writable);
        assert!(instruction.accounts[0].is_signer);
        // user_ata (writable, not signer)
        assert_eq!(instruction.accounts[1].pubkey, user_ata);
        assert!(instruction.accounts[1].is_writable);
        assert!(!instruction.accounts[1].is_signer);
        // eata (readonly, not signer)
        assert_eq!(instruction.accounts[2].pubkey, eata);
        assert!(!instruction.accounts[2].is_writable);
        assert!(!instruction.accounts[2].is_signer);
        // MAGIC_CONTEXT_ID (writable, not signer)
        assert_eq!(instruction.accounts[3].pubkey, MAGIC_CONTEXT_ID);
        assert!(instruction.accounts[3].is_writable);
        assert!(!instruction.accounts[3].is_signer);
        // MAGIC_PROGRAM_ID (readonly, not signer)
        assert_eq!(instruction.accounts[4].pubkey, MAGIC_PROGRAM_ID);
        assert!(!instruction.accounts[4].is_writable);
        assert!(!instruction.accounts[4].is_signer);

        assert_eq!(
            instruction.data[0],
            EphemeralSplDiscriminator::UndelegateEphemeralAta as u8
        );
    }

    #[test]
    fn test_create_ephemeral_ata_permission() {
        let eata = Pubkey::new_unique();
        let permission = Pubkey::new_unique();
        let payer = Pubkey::new_unique();
        let eata_bump = 255u8;
        let flag_byte = 0u8;

        let instruction =
            create_ephemeral_ata_permission(eata, permission, payer, eata_bump, flag_byte);

        assert_eq!(instruction.program_id, ESPL_TOKEN_PROGRAM_ID);
        assert_eq!(instruction.accounts.len(), 5);
        // eata (writable, not signer)
        assert_eq!(instruction.accounts[0].pubkey, eata);
        assert!(instruction.accounts[0].is_writable);
        assert!(!instruction.accounts[0].is_signer);
        // permission (writable, not signer)
        assert_eq!(instruction.accounts[1].pubkey, permission);
        assert!(instruction.accounts[1].is_writable);
        assert!(!instruction.accounts[1].is_signer);
        // payer (writable, signer)
        assert_eq!(instruction.accounts[2].pubkey, payer);
        assert!(instruction.accounts[2].is_writable);
        assert!(instruction.accounts[2].is_signer);
        // system_program (readonly)
        assert_eq!(instruction.accounts[3].pubkey, system_program::id());
        assert!(!instruction.accounts[3].is_writable);
        assert!(!instruction.accounts[3].is_signer);
        // PERMISSION_PROGRAM_ID (readonly)
        assert_eq!(instruction.accounts[4].pubkey, PERMISSION_PROGRAM_ID);
        assert!(!instruction.accounts[4].is_writable);
        assert!(!instruction.accounts[4].is_signer);
    }

    #[test]
    fn test_delegate_ephemeral_ata_permission() {
        let payer = Pubkey::new_unique();
        let eata = Pubkey::new_unique();
        let permission = Pubkey::new_unique();
        let system_prog = Pubkey::new_unique();
        let delegation_buffer = Pubkey::new_unique();
        let delegation_record = Pubkey::new_unique();
        let delegation_metadata = Pubkey::new_unique();
        let validator = Pubkey::new_unique();
        let eata_bump = 255u8;

        let instruction = delegate_ephemeral_ata_permission(
            payer,
            eata,
            permission,
            system_prog,
            delegation_buffer,
            delegation_record,
            delegation_metadata,
            validator,
            eata_bump,
        );

        assert_eq!(instruction.program_id, ESPL_TOKEN_PROGRAM_ID);
        assert_eq!(instruction.accounts.len(), 10);
        // payer (writable, signer)
        assert_eq!(instruction.accounts[0].pubkey, payer);
        assert!(instruction.accounts[0].is_writable);
        assert!(instruction.accounts[0].is_signer);
        // eata (writable, not signer)
        assert_eq!(instruction.accounts[1].pubkey, eata);
        assert!(instruction.accounts[1].is_writable);
        assert!(!instruction.accounts[1].is_signer);
        // PERMISSION_PROGRAM_ID (readonly)
        assert_eq!(instruction.accounts[2].pubkey, PERMISSION_PROGRAM_ID);
        assert!(!instruction.accounts[2].is_writable);
        // permission (writable, not signer)
        assert_eq!(instruction.accounts[3].pubkey, permission);
        assert!(instruction.accounts[3].is_writable);
        assert!(!instruction.accounts[3].is_signer);
        // system_program (readonly)
        assert_eq!(instruction.accounts[4].pubkey, system_prog);
        assert!(!instruction.accounts[4].is_writable);
        // delegation_buffer (writable)
        assert_eq!(instruction.accounts[5].pubkey, delegation_buffer);
        assert!(instruction.accounts[5].is_writable);
        // delegation_record (writable)
        assert_eq!(instruction.accounts[6].pubkey, delegation_record);
        assert!(instruction.accounts[6].is_writable);
        // delegation_metadata (writable)
        assert_eq!(instruction.accounts[7].pubkey, delegation_metadata);
        assert!(instruction.accounts[7].is_writable);
        // delegation_program (readonly)
        assert!(!instruction.accounts[8].is_writable);
        // validator (readonly)
        assert_eq!(instruction.accounts[9].pubkey, validator);
        assert!(!instruction.accounts[9].is_writable);
    }

    #[test]
    fn test_undelegate_ephemeral_ata_permission() {
        let payer = Pubkey::new_unique();
        let eata = Pubkey::new_unique();
        let permission = Pubkey::new_unique();

        let instruction = undelegate_ephemeral_ata_permission(payer, eata, permission);

        assert_eq!(instruction.program_id, ESPL_TOKEN_PROGRAM_ID);
        assert_eq!(instruction.accounts.len(), 6);
        // payer (readonly, signer)
        assert_eq!(instruction.accounts[0].pubkey, payer);
        assert!(!instruction.accounts[0].is_writable);
        assert!(instruction.accounts[0].is_signer);
        // eata (readonly, not signer)
        assert_eq!(instruction.accounts[1].pubkey, eata);
        assert!(!instruction.accounts[1].is_writable);
        assert!(!instruction.accounts[1].is_signer);
        // permission (writable, not signer)
        assert_eq!(instruction.accounts[2].pubkey, permission);
        assert!(instruction.accounts[2].is_writable);
        assert!(!instruction.accounts[2].is_signer);
        // PERMISSION_PROGRAM_ID (readonly)
        assert_eq!(instruction.accounts[3].pubkey, PERMISSION_PROGRAM_ID);
        assert!(!instruction.accounts[3].is_writable);
        // MAGIC_PROGRAM_ID (readonly)
        assert_eq!(instruction.accounts[4].pubkey, MAGIC_PROGRAM_ID);
        assert!(!instruction.accounts[4].is_writable);
        // MAGIC_CONTEXT_ID (writable, not signer)
        assert_eq!(instruction.accounts[5].pubkey, MAGIC_CONTEXT_ID);
        assert!(instruction.accounts[5].is_writable);
        assert!(!instruction.accounts[5].is_signer);
    }

    #[test]
    fn test_reset_ephemeral_ata_permission() {
        let eata = Pubkey::new_unique();
        let permission = Pubkey::new_unique();
        let owner = Pubkey::new_unique();
        let bump = 255u8;
        let flag_byte = 0u8;

        let instruction = reset_ephemeral_ata_permission(eata, permission, owner, bump, flag_byte);

        assert_eq!(instruction.program_id, ESPL_TOKEN_PROGRAM_ID);
        assert_eq!(instruction.accounts.len(), 4);
        // eata (writable, not signer)
        assert_eq!(instruction.accounts[0].pubkey, eata);
        assert!(instruction.accounts[0].is_writable);
        assert!(!instruction.accounts[0].is_signer);
        // permission (writable, not signer)
        assert_eq!(instruction.accounts[1].pubkey, permission);
        assert!(instruction.accounts[1].is_writable);
        assert!(!instruction.accounts[1].is_signer);
        // owner (readonly, signer)
        assert_eq!(instruction.accounts[2].pubkey, owner);
        assert!(!instruction.accounts[2].is_writable);
        assert!(instruction.accounts[2].is_signer);
        // PERMISSION_PROGRAM_ID (readonly)
        assert_eq!(instruction.accounts[3].pubkey, PERMISSION_PROGRAM_ID);
        assert!(!instruction.accounts[3].is_writable);
        assert!(!instruction.accounts[3].is_signer);
    }
}
