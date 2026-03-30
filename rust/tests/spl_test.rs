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
                AllocateTransferQueueBuilder, CreateEphemeralAtaPermissionBuilder,
                DelegateEphemeralAtaBuilder, DelegateEphemeralAtaPermissionBuilder,
                DelegateTransferQueueBuilder,
                DepositAndDelegateShuttleEphemeralAtaWithMergeAndPrivateTransferBuilder,
                DepositAndDelegateShuttleEphemeralAtaWithMergeAndPrivateTransferBuilderError,
                DepositAndQueueTransferBuilder, DepositSplTokensBuilder,
                EnsureTransferQueueCrankBuilder, InitializeEphemeralAtaBuilder,
                InitializeGlobalVaultBuilder, InitializeTransferQueueBuilder,
                LamportsDelegatedTransferBuilder, ResetEphemeralAtaPermissionBuilder,
                UndelegateAndCloseShuttleEphemeralAtaBuilder, UndelegateEphemeralAtaBuilder,
                UndelegateEphemeralAtaPermissionBuilder, WithdrawSplTokensBuilder,
            },
            find_lamports_pda, find_rent_pda, find_shuttle_ata, find_shuttle_ephemeral_ata,
            find_shuttle_wallet_ata, find_transfer_queue, find_vault_ata, EphemeralAta,
            EphemeralSplDiscriminator, GlobalVault,
        },
    };
    use magicblock_magic_program_api::Pubkey;
    #[cfg(feature = "encryption")]
    use sdk::signature::Keypair;
    #[cfg(feature = "encryption")]
    use sdk::signer::Signer;
    use solana_system_interface::program as system_program;
    use spl_associated_token_account_interface::address::get_associated_token_address;

    #[test]
    fn test_initialize_global_vault() {
        let payer = Pubkey::new_unique();
        let mint = Pubkey::new_unique();
        let (vault, _vault_bump) = GlobalVault::find_pda(&mint);
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
        assert_eq!(instruction.data.len(), 1);
    }

    #[test]
    fn test_initialize_ephemeral_ata() {
        let payer = Pubkey::new_unique();
        let user = Pubkey::new_unique();
        let mint = Pubkey::new_unique();
        let (eata, _eata_bump) = EphemeralAta::find_pda(&user, &mint);

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
        assert_eq!(instruction.data.len(), 1);
    }

    #[test]
    fn test_initialize_transfer_queue() {
        let payer = Pubkey::new_unique();
        let mint = Pubkey::new_unique();
        let validator = Pubkey::new_unique();
        let (queue, _queue_bump) = find_transfer_queue(&mint, &validator);
        let (queue_permission, _permission_bump) = Permission::find_pda(&queue);

        let instruction = InitializeTransferQueueBuilder {
            payer,
            mint,
            validator,
            requested_items: Some(92),
        }
        .instruction();

        assert_eq!(instruction.program_id, ESPL_TOKEN_PROGRAM_ID);
        assert_eq!(instruction.accounts.len(), 7);
        assert_eq!(instruction.accounts[0].pubkey, payer);
        assert_eq!(instruction.accounts[1].pubkey, queue);
        assert_eq!(instruction.accounts[2].pubkey, queue_permission);
        assert_eq!(instruction.accounts[3].pubkey, mint);
        assert_eq!(instruction.accounts[4].pubkey, validator);
        assert_eq!(instruction.accounts[5].pubkey, system_program::id());
        assert_eq!(instruction.accounts[6].pubkey, PERMISSION_PROGRAM_ID);
        assert_eq!(
            instruction.data[0],
            EphemeralSplDiscriminator::InitializeTransferQueue as u8
        );
        assert_eq!(&instruction.data[1..], &92_u32.to_le_bytes());
    }

    #[test]
    fn test_allocate_transfer_queue() {
        let queue = Pubkey::new_unique();
        let instruction = AllocateTransferQueueBuilder { queue }.instruction();

        assert_eq!(instruction.program_id, ESPL_TOKEN_PROGRAM_ID);
        assert_eq!(instruction.accounts.len(), 2);
        assert_eq!(instruction.accounts[0].pubkey, queue);
        assert_eq!(instruction.accounts[1].pubkey, system_program::id());
        assert_eq!(
            instruction.data,
            vec![EphemeralSplDiscriminator::AllocateTransferQueue as u8]
        );
    }

    #[test]
    fn test_deposit_and_queue_transfer() {
        let queue = Pubkey::new_unique();
        let vault = Pubkey::new_unique();
        let mint = Pubkey::new_unique();
        let source = Pubkey::new_unique();
        let vault_ata = Pubkey::new_unique();
        let destination = Pubkey::new_unique();
        let owner = Pubkey::new_unique();

        let instruction = DepositAndQueueTransferBuilder {
            queue,
            vault,
            mint,
            source,
            vault_ata,
            destination,
            owner,
            reimbursement_token_info: None,
            amount: 25,
            min_delay_ms: 100,
            max_delay_ms: 300,
            split: 4,
        }
        .instruction()
        .unwrap();

        assert_eq!(instruction.program_id, ESPL_TOKEN_PROGRAM_ID);
        assert_eq!(instruction.accounts.len(), 9);
        assert_eq!(instruction.accounts[0].pubkey, queue);
        assert_eq!(instruction.accounts[1].pubkey, vault);
        assert_eq!(instruction.accounts[2].pubkey, mint);
        assert_eq!(instruction.accounts[3].pubkey, source);
        assert_eq!(instruction.accounts[4].pubkey, vault_ata);
        assert_eq!(instruction.accounts[5].pubkey, destination);
        assert_eq!(instruction.accounts[6].pubkey, owner);
        assert_eq!(instruction.accounts[7].pubkey, TOKEN_PROGRAM_ID);
        assert_eq!(instruction.accounts[8].pubkey, source);
        assert_eq!(
            instruction.data[0],
            EphemeralSplDiscriminator::DepositAndQueueTransfer as u8
        );
        assert_eq!(
            u64::from_le_bytes(instruction.data[1..9].try_into().unwrap()),
            25
        );
        assert_eq!(
            u64::from_le_bytes(instruction.data[9..17].try_into().unwrap()),
            100
        );
        assert_eq!(
            u64::from_le_bytes(instruction.data[17..25].try_into().unwrap()),
            300
        );
        assert_eq!(
            u32::from_le_bytes(instruction.data[25..29].try_into().unwrap()),
            4
        );
    }

    #[test]
    fn test_deposit_and_queue_transfer_with_reimbursement_token_override() {
        let reimbursement_token_info = Pubkey::new_unique();

        let instruction = DepositAndQueueTransferBuilder {
            queue: Pubkey::new_unique(),
            vault: Pubkey::new_unique(),
            mint: Pubkey::new_unique(),
            source: Pubkey::new_unique(),
            vault_ata: Pubkey::new_unique(),
            destination: Pubkey::new_unique(),
            owner: Pubkey::new_unique(),
            reimbursement_token_info: Some(reimbursement_token_info),
            amount: 25,
            min_delay_ms: 100,
            max_delay_ms: 300,
            split: 4,
        }
        .instruction()
        .unwrap();

        assert_eq!(instruction.accounts[8].pubkey, reimbursement_token_info);
    }

    #[test]
    fn test_ensure_transfer_queue_crank() {
        let payer = Pubkey::new_unique();
        let queue = Pubkey::new_unique();
        let magic_fee_vault = Pubkey::new_unique();

        let instruction = EnsureTransferQueueCrankBuilder {
            payer,
            queue,
            magic_fee_vault,
            magic_context: MAGIC_CONTEXT_ID,
            magic_program: MAGIC_PROGRAM_ID,
        }
        .instruction();

        assert_eq!(instruction.program_id, ESPL_TOKEN_PROGRAM_ID);
        assert_eq!(instruction.accounts.len(), 5);
        assert_eq!(instruction.accounts[0].pubkey, payer);
        assert_eq!(instruction.accounts[1].pubkey, queue);
        assert_eq!(instruction.accounts[2].pubkey, magic_fee_vault);
        assert_eq!(instruction.accounts[3].pubkey, MAGIC_CONTEXT_ID);
        assert_eq!(instruction.accounts[4].pubkey, MAGIC_PROGRAM_ID);
        assert_eq!(
            instruction.data,
            vec![EphemeralSplDiscriminator::EnsureTransferQueueCrank as u8]
        );
    }

    #[test]
    fn test_lamports_delegated_transfer() {
        let payer = Pubkey::new_unique();
        let destination = Pubkey::new_unique();
        let salt = core::array::from_fn(|i| i as u8);
        let (rent_pda, _rent_bump) = find_rent_pda();
        let (lamports_pda, _lamports_bump) = find_lamports_pda(&payer, &destination, &salt);
        let destination_delegation_record =
            delegation_record_pda_from_delegated_account(&destination);

        let instruction = LamportsDelegatedTransferBuilder {
            payer,
            destination,
            amount: 25,
            salt,
        }
        .instruction();

        assert_eq!(instruction.program_id, ESPL_TOKEN_PROGRAM_ID);
        assert_eq!(instruction.accounts.len(), 11);
        assert_eq!(instruction.accounts[0].pubkey, payer);
        assert!(instruction.accounts[0].is_writable);
        assert!(instruction.accounts[0].is_signer);
        assert_eq!(instruction.accounts[1].pubkey, rent_pda);
        assert_eq!(instruction.accounts[2].pubkey, lamports_pda);
        assert_eq!(instruction.accounts[9].pubkey, destination);
        assert!(instruction.accounts[9].is_writable);
        assert_eq!(
            instruction.accounts[10].pubkey,
            destination_delegation_record
        );
        assert!(!instruction.accounts[10].is_writable);
        assert_eq!(
            instruction.data[0],
            EphemeralSplDiscriminator::LamportsDelegatedTransfer as u8
        );
        assert_eq!(
            u64::from_le_bytes(instruction.data[1..9].try_into().unwrap()),
            25
        );
        assert_eq!(&instruction.data[9..41], &salt);
    }

    #[test]
    fn test_delegate_transfer_queue() {
        let payer = Pubkey::new_unique();
        let queue = Pubkey::new_unique();
        let mint = Pubkey::new_unique();
        let delegation_buffer = delegate_buffer_pda_from_delegated_account_and_owner_program(
            &queue,
            &ESPL_TOKEN_PROGRAM_ID,
        );
        let delegation_record = delegation_record_pda_from_delegated_account(&queue);
        let delegation_metadata = delegation_metadata_pda_from_delegated_account(&queue);

        let instruction = DelegateTransferQueueBuilder { payer, queue, mint }.instruction();

        assert_eq!(instruction.program_id, ESPL_TOKEN_PROGRAM_ID);
        assert_eq!(instruction.accounts.len(), 9);
        assert_eq!(instruction.accounts[0].pubkey, payer);
        assert_eq!(instruction.accounts[1].pubkey, queue);
        assert_eq!(instruction.accounts[2].pubkey, mint);
        assert_eq!(instruction.accounts[3].pubkey, ESPL_TOKEN_PROGRAM_ID);
        assert_eq!(instruction.accounts[4].pubkey, delegation_buffer);
        assert_eq!(instruction.accounts[5].pubkey, delegation_record);
        assert_eq!(instruction.accounts[6].pubkey, delegation_metadata);
        assert_eq!(instruction.accounts[7].pubkey, DELEGATION_PROGRAM_ID);
        assert_eq!(instruction.accounts[8].pubkey, system_program::id());
        assert_eq!(
            instruction.data,
            vec![EphemeralSplDiscriminator::DelegateTransferQueue as u8]
        );
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
    fn test_undelegate_and_close_shuttle_ephemeral_ata() {
        let payer = Pubkey::new_unique();
        let rent_reimbursement = Pubkey::new_unique();
        let owner = Pubkey::new_unique();
        let mint = Pubkey::new_unique();
        let destination_ata = Pubkey::new_unique();
        let shuttle_id = 7;
        let (shuttle_ephemeral_ata, _shuttle_bump) =
            find_shuttle_ephemeral_ata(&owner, &mint, shuttle_id);
        let (shuttle_ata, _shuttle_ata_bump) = find_shuttle_ata(&shuttle_ephemeral_ata, &mint);
        let shuttle_wallet_ata = find_shuttle_wallet_ata(&mint, &shuttle_ephemeral_ata);

        let instruction = UndelegateAndCloseShuttleEphemeralAtaBuilder {
            payer,
            rent_reimbursement,
            owner,
            mint,
            destination_ata,
            shuttle_id,
            escrow_index: Some(3),
        }
        .instruction();

        assert_eq!(instruction.program_id, ESPL_TOKEN_PROGRAM_ID);
        assert_eq!(instruction.accounts.len(), 9);
        assert_eq!(instruction.accounts[0].pubkey, payer);
        assert_eq!(instruction.accounts[1].pubkey, rent_reimbursement);
        assert_eq!(instruction.accounts[2].pubkey, shuttle_ephemeral_ata);
        assert_eq!(instruction.accounts[3].pubkey, shuttle_ata);
        assert_eq!(instruction.accounts[4].pubkey, shuttle_wallet_ata);
        assert_eq!(instruction.accounts[5].pubkey, destination_ata);
        assert_eq!(instruction.accounts[6].pubkey, TOKEN_PROGRAM_ID);
        assert_eq!(instruction.accounts[7].pubkey, MAGIC_CONTEXT_ID);
        assert_eq!(instruction.accounts[8].pubkey, MAGIC_PROGRAM_ID);
        assert_eq!(
            instruction.data,
            vec![
                EphemeralSplDiscriminator::UndelegateAndCloseShuttleEphemeralAta as u8,
                3,
            ]
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

    #[cfg(feature = "encryption")]
    #[test]
    fn test_deposit_and_delegate_shuttle_private_transfer_requires_validator() {
        let instruction = DepositAndDelegateShuttleEphemeralAtaWithMergeAndPrivateTransferBuilder {
            payer: Pubkey::new_unique(),
            owner: Pubkey::new_unique(),
            mint: Pubkey::new_unique(),
            source_ata: Pubkey::new_unique(),
            destination_owner: Pubkey::new_unique(),
            shuttle_id: 7,
            amount: 25_u64,
            min_delay_ms: 100_u64,
            max_delay_ms: 300_u64,
            split: 4_u32,
            validator: None,
        }
        .instruction();

        assert!(matches!(
            instruction,
            Err(
                DepositAndDelegateShuttleEphemeralAtaWithMergeAndPrivateTransferBuilderError::MissingValidator
            )
        ));
    }

    #[cfg(feature = "encryption")]
    #[test]
    fn test_deposit_and_delegate_shuttle_private_transfer_instruction_layout() {
        let payer = Pubkey::new_unique();
        let owner = Pubkey::new_unique();
        let mint = Pubkey::new_unique();
        let source_ata = Pubkey::new_unique();
        let destination_owner = Pubkey::new_unique();
        let validator = Keypair::new().pubkey();
        let shuttle_id = 7;
        let amount = 25_u64;
        let min_delay_ms = 100_u64;
        let max_delay_ms = 300_u64;
        let split = 4_u32;

        let instruction = DepositAndDelegateShuttleEphemeralAtaWithMergeAndPrivateTransferBuilder {
            payer,
            owner,
            mint,
            source_ata,
            destination_owner,
            shuttle_id,
            amount,
            min_delay_ms,
            max_delay_ms,
            split,
            validator: Some(validator),
        }
        .instruction()
        .unwrap();

        let (rent_pda, _) = find_rent_pda();
        let (shuttle_ephemeral_ata, _) = find_shuttle_ephemeral_ata(&owner, &mint, shuttle_id);
        let (shuttle_ata, _) = find_shuttle_ata(&shuttle_ephemeral_ata, &mint);
        let shuttle_wallet_ata = find_shuttle_wallet_ata(&mint, &shuttle_ephemeral_ata);
        let (vault, _) = GlobalVault::find_pda(&mint);
        let vault_ata = find_vault_ata(&mint, &vault);
        let (queue, _) = find_transfer_queue(&mint, &validator);

        assert_eq!(instruction.program_id, ESPL_TOKEN_PROGRAM_ID);
        assert_eq!(instruction.accounts.len(), 19);
        assert_eq!(instruction.accounts[0].pubkey, payer);
        assert_eq!(instruction.accounts[1].pubkey, rent_pda);
        assert_eq!(instruction.accounts[2].pubkey, shuttle_ephemeral_ata);
        assert_eq!(instruction.accounts[3].pubkey, shuttle_ata);
        assert_eq!(instruction.accounts[4].pubkey, shuttle_wallet_ata);
        assert_eq!(instruction.accounts[5].pubkey, owner);
        assert_eq!(instruction.accounts[13].pubkey, mint);
        assert_eq!(instruction.accounts[16].pubkey, source_ata);
        assert_eq!(instruction.accounts[17].pubkey, vault_ata);
        assert_eq!(instruction.accounts[18].pubkey, queue);

        assert_eq!(
            instruction.data[0],
            EphemeralSplDiscriminator::DepositAndDelegateShuttleEphemeralAtaWithMergeAndPrivateTransfer
                as u8
        );
        assert_eq!(
            u32::from_le_bytes(instruction.data[1..5].try_into().unwrap()),
            shuttle_id
        );
        assert_eq!(
            u64::from_le_bytes(instruction.data[5..13].try_into().unwrap()),
            amount
        );

        let validator_len = instruction.data[13] as usize;
        assert_eq!(validator_len, 32);
        assert_eq!(&instruction.data[14..46], validator.as_ref());

        let destination_len = instruction.data[46] as usize;
        assert_eq!(destination_len, 80);

        let suffix_offset = 47 + destination_len;
        let suffix_len = instruction.data[suffix_offset] as usize;
        assert_eq!(suffix_len, 69);
        assert_eq!(suffix_offset + 1 + suffix_len, instruction.data.len());
    }
}
