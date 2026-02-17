// Tests for spl crate
// Tests the SPL vault and token account instructions

#[cfg(test)]
mod tests {
    use ephemeral_rollups_sdk::{
        consts::{ESPL_TOKEN_PROGRAM_ID, MAGIC_CONTEXT_ID, MAGIC_PROGRAM_ID, TOKEN_PROGRAM_ID},
        spl::instructions::*,
    };
    use solana_pubkey::Pubkey;

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
}
