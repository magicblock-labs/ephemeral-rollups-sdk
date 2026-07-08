use {
    crate::{consts::RENT_PENDING_ATA_CLOSE_AUTHORITY, spl::consts::ASSOCIATED_TOKEN_PROGRAM_ID},
    pinocchio::Address,
};

const TOKEN_ACCOUNT_CLOSE_AUTHORITY_OFFSET: usize = 129;
const TOKEN_ACCOUNT_CLOSE_AUTHORITY_PUBKEY_OFFSET: usize = 133;
const TOKEN_ACCOUNT_LEN: usize = 165;

pub fn rent_pending_ata_address(
    wallet_owner: &Address,
    mint: &Address,
    token_program: &Address,
) -> (Address, u8) {
    Address::find_program_address(
        &[wallet_owner.as_ref(), token_program.as_ref(), mint.as_ref()],
        &ASSOCIATED_TOKEN_PROGRAM_ID,
    )
}

pub fn is_rent_pending_token_account(data: &[u8]) -> bool {
    if data.len() < TOKEN_ACCOUNT_LEN {
        return false;
    }

    let close_authority_tag = u32::from_le_bytes([
        data[TOKEN_ACCOUNT_CLOSE_AUTHORITY_OFFSET],
        data[TOKEN_ACCOUNT_CLOSE_AUTHORITY_OFFSET + 1],
        data[TOKEN_ACCOUNT_CLOSE_AUTHORITY_OFFSET + 2],
        data[TOKEN_ACCOUNT_CLOSE_AUTHORITY_OFFSET + 3],
    ]);

    close_authority_tag == 1
        && &data[TOKEN_ACCOUNT_CLOSE_AUTHORITY_PUBKEY_OFFSET
            ..TOKEN_ACCOUNT_CLOSE_AUTHORITY_PUBKEY_OFFSET + 32]
            == RENT_PENDING_ATA_CLOSE_AUTHORITY.as_ref()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::spl::consts::TOKEN_PROGRAM_ID;

    #[test]
    fn test_rent_pending_ata_address() {
        let wallet_owner = Address::new_from_array([1; 32]);
        let mint = Address::new_from_array([2; 32]);
        let token_program = TOKEN_PROGRAM_ID;
        let (ata, _) = rent_pending_ata_address(&wallet_owner, &mint, &token_program);
        let (expected_ata, _) = Address::find_program_address(
            &[wallet_owner.as_ref(), token_program.as_ref(), mint.as_ref()],
            &crate::spl::consts::ASSOCIATED_TOKEN_PROGRAM_ID,
        );

        assert_eq!(ata, expected_ata);
    }

    #[test]
    fn test_is_rent_pending_token_account() {
        let mut data = [0u8; TOKEN_ACCOUNT_LEN];
        data[TOKEN_ACCOUNT_CLOSE_AUTHORITY_OFFSET..TOKEN_ACCOUNT_CLOSE_AUTHORITY_OFFSET + 4]
            .copy_from_slice(&1u32.to_le_bytes());
        data[TOKEN_ACCOUNT_CLOSE_AUTHORITY_PUBKEY_OFFSET
            ..TOKEN_ACCOUNT_CLOSE_AUTHORITY_PUBKEY_OFFSET + 32]
            .copy_from_slice(RENT_PENDING_ATA_CLOSE_AUTHORITY.as_ref());

        assert!(is_rent_pending_token_account(&data));
        data[TOKEN_ACCOUNT_CLOSE_AUTHORITY_OFFSET..TOKEN_ACCOUNT_CLOSE_AUTHORITY_OFFSET + 4]
            .copy_from_slice(&0u32.to_le_bytes());
        assert!(!is_rent_pending_token_account(&data));
        assert!(!is_rent_pending_token_account(
            &data[..TOKEN_ACCOUNT_LEN - 1]
        ));
    }
}
