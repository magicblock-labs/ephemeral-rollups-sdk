// Tests for spl crate
// Tests the SPL vault and token account instructions

#[cfg(test)]
mod tests {
    use dlp_api::{
        consts::DELEGATION_PROGRAM_ID,
        pda::{
            delegate_buffer_pda_from_delegated_account_and_owner_program,
            delegation_metadata_pda_from_delegated_account,
            delegation_record_pda_from_delegated_account,
        },
    };
    use ephemeral_rollups_sdk::{
        access_control::structs::Permission,
        consts::{
            ASSOCIATED_TOKEN_PROGRAM_ID, ESPL_TOKEN_PROGRAM_ID, MAGIC_CONTEXT_ID, MAGIC_PROGRAM_ID,
            PERMISSION_PROGRAM_ID, TOKEN_PROGRAM_ID,
        },
        spl::{
            builders::{
                CreateEphemeralAtaPermissionBuilder, DelegateEphemeralAtaBuilder,
                DelegateEphemeralAtaPermissionBuilder, DepositSplTokensBuilder,
                InitializeEphemeralAtaBuilder, InitializeGlobalVaultBuilder,
                ResetEphemeralAtaPermissionBuilder, UndelegateEphemeralAtaBuilder,
                UndelegateEphemeralAtaPermissionBuilder, WithdrawSplTokensBuilder,
            },
            EphemeralAta, EphemeralSplDiscriminator, GlobalVault,
        },
    };
    use magicblock_magic_program_api::Pubkey;
    use solana_system_interface::program as system_program;
    use spl_associated_token_account_interface::address::get_associated_token_address;

    #[test]
    fn test_initialize_global_vault() {
        let payer = Pubkey::new_unique();
        let mint = Pubkey::new_unique();
        let (vault, vault_bump) = GlobalVault::find_pda(&mint);
        let (vault_ephemeral_ata, _vault_eata_bump) = EphemeralAta::find_pda(&vault, &mint);
        let vault_ata = get_associated_token_address(&vault, &mint);

        let instruction = InitializeGlobalVaultBuilder { payer, mint }.instruction();

        assert_eq!(instruction.program_id, ESPL_TOKEN_PROGRAM_ID);
        assert_eq!(instruction.accounts.len(), 8);
        // vault (writable, not signer)
        assert_eq!(instruction.accounts[0].pubkey, vault);
        assert!(instruction.accounts[0].is_writable);
        assert!(!instruction.accounts[0].is_signer);
        // payer (writable, signer)
        assert_eq!(instruction.accounts[1].pubkey, payer);
        assert!(instruction.accounts[1].is_writable);
        assert!(instruction.accounts[1].is_signer);
        // mint (readonly)
        assert_eq!(instruction.accounts[2].pubkey, mint);
        assert!(!instruction.accounts[2].is_writable);
        assert!(!instruction.accounts[2].is_signer);
        // vault_ephemeral_ata (writable)
        assert_eq!(instruction.accounts[3].pubkey, vault_ephemeral_ata);
        assert!(instruction.accounts[3].is_writable);
        assert!(!instruction.accounts[3].is_signer);
        // vault_ata (writable)
        assert_eq!(instruction.accounts[4].pubkey, vault_ata);
        assert!(instruction.accounts[4].is_writable);
        assert!(!instruction.accounts[4].is_signer);
        // token_program (readonly)
        assert_eq!(instruction.accounts[5].pubkey, TOKEN_PROGRAM_ID);
        assert!(!instruction.accounts[5].is_writable);
        assert!(!instruction.accounts[5].is_signer);
        // associated_token_program (readonly)
        assert_eq!(instruction.accounts[6].pubkey, ASSOCIATED_TOKEN_PROGRAM_ID);
        assert!(!instruction.accounts[6].is_writable);
        assert!(!instruction.accounts[6].is_signer);
        // system_program (readonly)
        assert_eq!(instruction.accounts[7].pubkey, system_program::id());
        assert!(!instruction.accounts[7].is_writable);
        assert!(!instruction.accounts[7].is_signer);

        assert_eq!(
            instruction.data[0],
            EphemeralSplDiscriminator::InitializeGlobalVault as u8
        );
        assert_eq!(instruction.data[1], vault_bump);
    }

    #[test]
    fn test_initialize_ephemeral_ata() {
        let payer = Pubkey::new_unique();
        let user = Pubkey::new_unique();
        let mint = Pubkey::new_unique();
        let (eata, eata_bump) = EphemeralAta::find_pda(&user, &mint);

        let instruction = InitializeEphemeralAtaBuilder { payer, user, mint }.instruction();

        assert_eq!(instruction.program_id, ESPL_TOKEN_PROGRAM_ID);
        assert_eq!(instruction.accounts.len(), 5);
        // eata (writable, not signer)
        assert_eq!(instruction.accounts[0].pubkey, eata);
        assert!(instruction.accounts[0].is_writable);
        assert!(!instruction.accounts[0].is_signer);
        // payer (writable, not signer)
        assert_eq!(instruction.accounts[1].pubkey, payer);
        assert!(instruction.accounts[1].is_writable);
        assert!(instruction.accounts[1].is_signer);
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
        let mint = Pubkey::new_unique();
        let user = Pubkey::new_unique();
        let (eata, _eata_bump) = EphemeralAta::find_pda(&user, &mint);
        let (vault, _vault_bump) = GlobalVault::find_pda(&mint);
        let amount = 1000u64;

        let instruction = DepositSplTokensBuilder {
            authority,
            user,
            mint,
            amount,
        }
        .instruction();

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
        assert_eq!(
            instruction.accounts[3].pubkey,
            get_associated_token_address(&user, &mint)
        );
        assert!(instruction.accounts[3].is_writable);
        // vault_token_acc (writable)
        assert_eq!(
            instruction.accounts[4].pubkey,
            get_associated_token_address(&vault, &mint)
        );
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
        let mint = Pubkey::new_unique();
        let user = Pubkey::new_unique();
        let (eata, _eata_bump) = EphemeralAta::find_pda(&user, &mint);
        let (vault, _vault_bump) = GlobalVault::find_pda(&mint);
        let amount = 1000u64;

        let instruction = WithdrawSplTokensBuilder {
            payer,
            user,
            mint,
            amount,
        }
        .instruction();

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
        assert_eq!(
            instruction.accounts[3].pubkey,
            get_associated_token_address(&vault, &mint)
        );
        assert!(instruction.accounts[3].is_writable);
        // user_ata (writable)
        assert_eq!(
            instruction.accounts[4].pubkey,
            get_associated_token_address(&user, &mint)
        );
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
        assert_eq!(instruction.data.len(), 9);
    }

    #[test]
    fn test_delegate_ephemeral_ata() {
        let payer = Pubkey::new_unique();
        let user = Pubkey::new_unique();
        let mint = Pubkey::new_unique();
        let (eata, _eata_bump) = EphemeralAta::find_pda(&user, &mint);

        let instruction = DelegateEphemeralAtaBuilder {
            payer,
            user,
            mint,
            validator: None,
        }
        .instruction();

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
        assert_eq!(
            instruction.accounts[3].pubkey,
            delegate_buffer_pda_from_delegated_account_and_owner_program(
                &eata,
                &ESPL_TOKEN_PROGRAM_ID
            )
        );
        assert!(instruction.accounts[3].is_writable);
        // delegation_record (writable)
        assert_eq!(
            instruction.accounts[4].pubkey,
            delegation_record_pda_from_delegated_account(&eata)
        );
        assert!(instruction.accounts[4].is_writable);
        // delegation_metadata (writable)
        assert_eq!(
            instruction.accounts[5].pubkey,
            delegation_metadata_pda_from_delegated_account(&eata)
        );
        assert!(instruction.accounts[5].is_writable);

        assert_eq!(
            instruction.data[0],
            EphemeralSplDiscriminator::DelegateEphemeralAta as u8
        );
        assert_eq!(instruction.data.len(), 1);
    }

    #[test]
    fn test_delegate_ephemeral_ata_some_validator() {
        let payer = Pubkey::new_unique();
        let user = Pubkey::new_unique();
        let mint = Pubkey::new_unique();
        let validator = Pubkey::new_unique();
        let (eata, _eata_bump) = EphemeralAta::find_pda(&user, &mint);

        let instruction = DelegateEphemeralAtaBuilder {
            payer,
            user,
            mint,
            validator: Some(validator),
        }
        .instruction();

        assert_eq!(instruction.program_id, ESPL_TOKEN_PROGRAM_ID);
        assert_eq!(instruction.accounts.len(), 8);
        // payer (writable, signer)
        assert_eq!(instruction.accounts[0].pubkey, payer);
        assert!(instruction.accounts[0].is_writable);
        assert!(instruction.accounts[0].is_signer);
        // eata (writable)
        assert_eq!(
            instruction.accounts[1].pubkey,
            EphemeralAta::find_pda(&user, &mint).0
        );
        assert!(instruction.accounts[1].is_writable);
        // ESPL_TOKEN_PROGRAM_ID (readonly)
        assert_eq!(instruction.accounts[2].pubkey, ESPL_TOKEN_PROGRAM_ID);
        assert!(!instruction.accounts[2].is_writable);
        // delegation_buffer (writable)
        assert_eq!(
            instruction.accounts[3].pubkey,
            delegate_buffer_pda_from_delegated_account_and_owner_program(
                &eata,
                &ESPL_TOKEN_PROGRAM_ID
            )
        );
        assert!(instruction.accounts[3].is_writable);
        // delegation_record (writable)
        assert_eq!(
            instruction.accounts[4].pubkey,
            delegation_record_pda_from_delegated_account(&eata)
        );
        assert!(instruction.accounts[4].is_writable);
        // delegation_metadata (writable)
        assert_eq!(
            instruction.accounts[5].pubkey,
            delegation_metadata_pda_from_delegated_account(&eata)
        );
        assert!(instruction.accounts[5].is_writable);
        // delegation_program (readonly)
        assert_eq!(instruction.accounts[6].pubkey, DELEGATION_PROGRAM_ID);
        assert!(!instruction.accounts[6].is_writable);
        // system_program (readonly)
        assert_eq!(instruction.accounts[7].pubkey, system_program::id());
        assert!(!instruction.accounts[7].is_writable);
        assert!(!instruction.accounts[7].is_signer);

        assert_eq!(
            instruction.data[0],
            EphemeralSplDiscriminator::DelegateEphemeralAta as u8
        );
        assert_eq!(instruction.data[1..33], validator.to_bytes());
    }

    #[test]
    fn test_undelegate_ephemeral_ata() {
        let payer = Pubkey::new_unique();
        let user = Pubkey::new_unique();
        let mint = Pubkey::new_unique();

        let instruction = UndelegateEphemeralAtaBuilder { payer, user, mint }.instruction();

        assert_eq!(instruction.program_id, ESPL_TOKEN_PROGRAM_ID);
        assert_eq!(instruction.accounts.len(), 5);
        // payer (readonly, signer)
        assert_eq!(instruction.accounts[0].pubkey, payer);
        assert!(!instruction.accounts[0].is_writable);
        assert!(instruction.accounts[0].is_signer);
        // user_ata (writable, not signer)
        assert_eq!(
            instruction.accounts[1].pubkey,
            get_associated_token_address(&user, &mint)
        );
        assert!(instruction.accounts[1].is_writable);
        assert!(!instruction.accounts[1].is_signer);
        // eata (readonly, not signer)
        assert_eq!(
            instruction.accounts[2].pubkey,
            EphemeralAta::find_pda(&user, &mint).0
        );
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
        let payer = Pubkey::new_unique();
        let user = Pubkey::new_unique();
        let mint = Pubkey::new_unique();
        let (eata, _eata_bump) = EphemeralAta::find_pda(&user, &mint);
        let (permission, _permission_bump) = Permission::find_pda(&eata);
        let flag_byte = 0u8;

        let instruction = CreateEphemeralAtaPermissionBuilder {
            payer,
            user,
            mint,
            flag_byte,
        }
        .instruction();

        assert_eq!(instruction.program_id, ESPL_TOKEN_PROGRAM_ID);
        assert_eq!(instruction.accounts.len(), 5);
        // eata (writable, not signer)
        assert_eq!(
            instruction.accounts[0].pubkey,
            EphemeralAta::find_pda(&user, &mint).0
        );
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

        assert_eq!(
            instruction.data[0],
            EphemeralSplDiscriminator::CreateEphemeralAtaPermission as u8
        );
        assert_eq!(instruction.data[1], flag_byte);
    }

    #[test]
    fn test_delegate_ephemeral_ata_permission() {
        let payer = Pubkey::new_unique();
        let user = Pubkey::new_unique();
        let mint = Pubkey::new_unique();
        let (eata, _eata_bump) = EphemeralAta::find_pda(&user, &mint);
        let (permission, _permission_bump) = Permission::find_pda(&eata);
        let validator = Pubkey::new_unique();

        let instruction = DelegateEphemeralAtaPermissionBuilder {
            payer,
            user,
            mint,
            validator,
        }
        .instruction();

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
        assert_eq!(instruction.accounts[4].pubkey, system_program::id());
        assert!(!instruction.accounts[4].is_writable);
        // delegation_buffer (writable)
        assert_eq!(
            instruction.accounts[5].pubkey,
            delegate_buffer_pda_from_delegated_account_and_owner_program(
                &permission,
                &PERMISSION_PROGRAM_ID
            )
        );
        assert!(instruction.accounts[5].is_writable);
        // delegation_record (writable)
        assert_eq!(
            instruction.accounts[6].pubkey,
            delegation_record_pda_from_delegated_account(&permission)
        );
        assert!(instruction.accounts[6].is_writable);
        // delegation_metadata (writable)
        assert_eq!(
            instruction.accounts[7].pubkey,
            delegation_metadata_pda_from_delegated_account(&permission)
        );
        assert!(instruction.accounts[7].is_writable);
        // delegation_program (readonly)
        assert_eq!(instruction.accounts[8].pubkey, DELEGATION_PROGRAM_ID);
        assert!(!instruction.accounts[8].is_writable);
        // validator (readonly)
        assert_eq!(instruction.accounts[9].pubkey, validator);
        assert!(!instruction.accounts[9].is_writable);

        assert_eq!(
            instruction.data[0],
            EphemeralSplDiscriminator::DelegateEphemeralAtaPermission as u8
        );
        assert_eq!(instruction.data.len(), 1);
    }

    #[test]
    fn test_undelegate_ephemeral_ata_permission() {
        let payer = Pubkey::new_unique();
        let user = Pubkey::new_unique();
        let mint = Pubkey::new_unique();
        let (eata, _eata_bump) = EphemeralAta::find_pda(&user, &mint);
        let (permission, _permission_bump) = Permission::find_pda(&eata);

        let instruction =
            UndelegateEphemeralAtaPermissionBuilder { payer, user, mint }.instruction();

        assert_eq!(instruction.program_id, ESPL_TOKEN_PROGRAM_ID);
        assert_eq!(instruction.accounts.len(), 6);
        // payer (readonly, signer)
        assert_eq!(instruction.accounts[0].pubkey, payer);
        assert!(!instruction.accounts[0].is_writable);
        assert!(instruction.accounts[0].is_signer);
        // eata (readonly, not signer)
        assert_eq!(
            instruction.accounts[1].pubkey,
            EphemeralAta::find_pda(&user, &mint).0
        );
        assert!(instruction.accounts[1].is_writable);
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

        assert_eq!(
            instruction.data[0],
            EphemeralSplDiscriminator::UndelegateEphemeralAtaPermission as u8
        );
    }

    #[test]
    fn test_reset_ephemeral_ata_permission() {
        let user = Pubkey::new_unique();
        let mint = Pubkey::new_unique();
        let (eata, _eata_bump) = EphemeralAta::find_pda(&user, &mint);
        let (permission, _) = Permission::find_pda(&eata);
        let flag_byte = 0u8;

        let instruction = ResetEphemeralAtaPermissionBuilder {
            user,
            mint,
            flag_byte,
        }
        .instruction();

        assert_eq!(instruction.program_id, ESPL_TOKEN_PROGRAM_ID);
        assert_eq!(instruction.accounts.len(), 4);
        // eata (readonly, not signer)
        assert_eq!(
            instruction.accounts[0].pubkey,
            EphemeralAta::find_pda(&user, &mint).0
        );
        assert!(!instruction.accounts[0].is_writable);
        assert!(!instruction.accounts[0].is_signer);
        // permission (writable, not signer)
        assert_eq!(instruction.accounts[1].pubkey, permission);
        assert!(instruction.accounts[1].is_writable);
        assert!(!instruction.accounts[1].is_signer);
        // owner (readonly, signer)
        assert_eq!(instruction.accounts[2].pubkey, user);
        assert!(!instruction.accounts[2].is_writable);
        assert!(instruction.accounts[2].is_signer);
        // PERMISSION_PROGRAM_ID (readonly)
        assert_eq!(instruction.accounts[3].pubkey, PERMISSION_PROGRAM_ID);
        assert!(!instruction.accounts[3].is_writable);
        assert!(!instruction.accounts[3].is_signer);

        assert_eq!(
            instruction.data[0],
            EphemeralSplDiscriminator::ResetEphemeralAtaPermission as u8
        );
        assert_eq!(instruction.data[1], flag_byte);
    }
}
