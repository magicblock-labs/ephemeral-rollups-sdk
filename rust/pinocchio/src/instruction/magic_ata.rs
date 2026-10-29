use {
    crate::{consts::MAGIC_ATA_CLOSE_AUTHORITY, spl::consts::ASSOCIATED_TOKEN_PROGRAM_ID},
    pinocchio::{
        cpi::{invoke_signed_with_bounds, Signer},
        instruction::{InstructionAccount, InstructionView},
        AccountView, Address, ProgramResult,
    },
};

pub const CREATE_MAGIC_ATA_DISCRIMINATOR: u32 = 15;
pub const CREATE_MAGIC_ATA_DATA_LEN: usize = 36;
pub const CLOSE_MAGIC_ATA_DISCRIMINATOR: u32 = 26;

const TOKEN_ACCOUNT_CLOSE_AUTHORITY_OFFSET: usize = 129;
const TOKEN_ACCOUNT_CLOSE_AUTHORITY_PUBKEY_OFFSET: usize = 133;
const TOKEN_ACCOUNT_LEN: usize = 165;

pub fn get_associated_token_address(
    wallet_owner: &Address,
    mint: &Address,
    token_program: &Address,
) -> (Address, u8) {
    Address::find_program_address(
        &[wallet_owner.as_ref(), token_program.as_ref(), mint.as_ref()],
        &ASSOCIATED_TOKEN_PROGRAM_ID,
    )
}

pub fn encode_create_magic_ata_data(
    data: &mut [u8; CREATE_MAGIC_ATA_DATA_LEN],
    wallet_owner: &Address,
) {
    data[0..4].copy_from_slice(&CREATE_MAGIC_ATA_DISCRIMINATOR.to_le_bytes());
    data[4..36].copy_from_slice(wallet_owner.as_ref());
}

pub fn is_magic_ata_token_account(data: &[u8]) -> bool {
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
            == MAGIC_ATA_CLOSE_AUTHORITY.as_ref()
}

/// Create a Magic ATA through the Magic Program.
///
/// Requires a validator that supports Magic ATA materialization.
///
/// Idempotent on the validator side: succeeds if the ATA already exists as a
/// Magic ATA or projected token account for the same wallet owner and mint.
/// The ATA must end the transaction with a positive token amount, otherwise
/// the whole transaction is rolled back by the validator.
pub struct CreateMagicAta<'a> {
    pub payer: &'a AccountView,
    pub ata: &'a AccountView,
    pub mint: &'a AccountView,
    pub token_program: &'a AccountView,
    pub magic_program: &'a AccountView,
    pub wallet_owner: &'a Address,
}

impl<'a> CreateMagicAta<'a> {
    #[inline(always)]
    pub fn invoke(&self) -> ProgramResult {
        self.invoke_signed(&[])
    }

    #[inline(always)]
    pub fn invoke_signed(&self, signers: &[Signer<'_, '_>]) -> ProgramResult {
        const NUM_ACCOUNTS: usize = 4;

        let instruction_accounts = [
            InstructionAccount::readonly_signer(self.payer.address()),
            InstructionAccount::writable(self.ata.address()),
            InstructionAccount::readonly(self.mint.address()),
            InstructionAccount::readonly(self.token_program.address()),
        ];
        let accounts: [&AccountView; NUM_ACCOUNTS] =
            [self.payer, self.ata, self.mint, self.token_program];

        let mut instruction_data = [0u8; CREATE_MAGIC_ATA_DATA_LEN];
        encode_create_magic_ata_data(&mut instruction_data, self.wallet_owner);

        invoke_signed_with_bounds::<NUM_ACCOUNTS, _>(
            &InstructionView {
                program_id: self.magic_program.address(),
                accounts: &instruction_accounts,
                data: &instruction_data,
            },
            &accounts,
            signers,
        )
    }
}

/// Close a drained Magic ATA through the Magic Program.
///
/// No-op unless the ATA matches the Magic ATA marker for the signing owner
/// and holds zero tokens, so it can be appended unconditionally to withdrawal
/// flows.
pub struct CloseMagicAta<'a> {
    pub owner: &'a AccountView,
    pub ata: &'a AccountView,
    pub magic_program: &'a AccountView,
}

impl CloseMagicAta<'_> {
    #[inline(always)]
    pub fn invoke(&self) -> ProgramResult {
        self.invoke_signed(&[])
    }

    #[inline(always)]
    pub fn invoke_signed(&self, signers: &[Signer<'_, '_>]) -> ProgramResult {
        const NUM_ACCOUNTS: usize = 2;

        let instruction_accounts = [
            InstructionAccount::readonly_signer(self.owner.address()),
            InstructionAccount::writable(self.ata.address()),
        ];
        let accounts: [&AccountView; NUM_ACCOUNTS] = [self.owner, self.ata];

        invoke_signed_with_bounds::<NUM_ACCOUNTS, _>(
            &InstructionView {
                program_id: self.magic_program.address(),
                accounts: &instruction_accounts,
                data: &CLOSE_MAGIC_ATA_DISCRIMINATOR.to_le_bytes(),
            },
            &accounts,
            signers,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::spl::consts::TOKEN_PROGRAM_ID;

    #[test]
    fn test_encode_create_magic_ata_data() {
        let wallet_owner = Address::new_from_array([1; 32]);
        let mut data = [0u8; CREATE_MAGIC_ATA_DATA_LEN];

        encode_create_magic_ata_data(&mut data, &wallet_owner);

        assert_eq!(
            u32::from_le_bytes(data[0..4].try_into().unwrap()),
            CREATE_MAGIC_ATA_DISCRIMINATOR
        );
        assert_eq!(&data[4..36], wallet_owner.as_ref());
    }

    #[test]
    fn test_get_associated_token_address() {
        let wallet_owner = Address::new_from_array([1; 32]);
        let mint = Address::new_from_array([2; 32]);
        let token_program = TOKEN_PROGRAM_ID;
        let (ata, _) = get_associated_token_address(&wallet_owner, &mint, &token_program);
        let (expected_ata, _) = Address::find_program_address(
            &[wallet_owner.as_ref(), token_program.as_ref(), mint.as_ref()],
            &crate::spl::consts::ASSOCIATED_TOKEN_PROGRAM_ID,
        );

        assert_eq!(ata, expected_ata);
    }

    #[test]
    fn test_is_magic_ata_token_account() {
        let mut data = [0u8; TOKEN_ACCOUNT_LEN];
        data[TOKEN_ACCOUNT_CLOSE_AUTHORITY_OFFSET..TOKEN_ACCOUNT_CLOSE_AUTHORITY_OFFSET + 4]
            .copy_from_slice(&1u32.to_le_bytes());
        data[TOKEN_ACCOUNT_CLOSE_AUTHORITY_PUBKEY_OFFSET
            ..TOKEN_ACCOUNT_CLOSE_AUTHORITY_PUBKEY_OFFSET + 32]
            .copy_from_slice(MAGIC_ATA_CLOSE_AUTHORITY.as_ref());

        assert!(is_magic_ata_token_account(&data));
        data[TOKEN_ACCOUNT_CLOSE_AUTHORITY_OFFSET..TOKEN_ACCOUNT_CLOSE_AUTHORITY_OFFSET + 4]
            .copy_from_slice(&0u32.to_le_bytes());
        assert!(!is_magic_ata_token_account(&data));
        assert!(!is_magic_ata_token_account(&data[..TOKEN_ACCOUNT_LEN - 1]));
    }
}
