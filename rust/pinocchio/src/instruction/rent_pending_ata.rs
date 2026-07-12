use {
    crate::{consts::RENT_PENDING_ATA_CLOSE_AUTHORITY, spl::consts::ASSOCIATED_TOKEN_PROGRAM_ID},
    core::{mem::MaybeUninit, slice::from_raw_parts},
    pinocchio::{
        cpi::{invoke_signed_with_bounds, Signer},
        instruction::{InstructionAccount, InstructionView},
        AccountView, Address, ProgramResult,
    },
};

pub const CREATE_RENT_PENDING_ATA_DISCRIMINATOR: u32 = 15;
pub const CREATE_RENT_PENDING_ATA_DATA_LEN: usize = 100;

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

pub fn encode_create_rent_pending_ata_data(
    data: &mut [u8; CREATE_RENT_PENDING_ATA_DATA_LEN],
    wallet_owner: &Address,
    mint: &Address,
    token_program: &Address,
) {
    data[0..4].copy_from_slice(&CREATE_RENT_PENDING_ATA_DISCRIMINATOR.to_le_bytes());
    data[4..36].copy_from_slice(wallet_owner.as_ref());
    data[36..68].copy_from_slice(mint.as_ref());
    data[68..100].copy_from_slice(token_program.as_ref());
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

/// Create a rent-pending ATA through the Magic Program.
///
/// Requires a validator that supports rent-pending ATA materialization.
///
/// Idempotent on the validator side: succeeds if the ATA already exists as a
/// rent-pending or projected token account for the same wallet owner and mint.
/// The ATA must end the transaction with a positive token amount, otherwise
/// the whole transaction is rolled back by the validator.
pub struct CreateRentPendingAta<'a> {
    pub payer: &'a AccountView,
    pub ata: &'a AccountView,
    pub mint: &'a AccountView,
    pub token_program: &'a AccountView,
    pub magic_program: &'a AccountView,
    pub wallet_owner: &'a Address,
}

impl<'a> CreateRentPendingAta<'a> {
    #[inline(always)]
    pub fn invoke(&self) -> ProgramResult {
        self.invoke_signed(&[])
    }

    #[inline(always)]
    pub fn invoke_signed(&self, signers: &[Signer<'_, '_>]) -> ProgramResult {
        const NUM_ACCOUNTS: usize = 4;

        let mut instruction_accounts =
            [const { MaybeUninit::<InstructionAccount>::uninit() }; NUM_ACCOUNTS];
        instruction_accounts[0].write(InstructionAccount::readonly_signer(self.payer.address()));
        instruction_accounts[1].write(InstructionAccount::writable(self.ata.address()));
        instruction_accounts[2].write(InstructionAccount::readonly(self.mint.address()));
        instruction_accounts[3].write(InstructionAccount::readonly(self.token_program.address()));

        let mut accounts = [const { MaybeUninit::<&AccountView>::uninit() }; NUM_ACCOUNTS];
        accounts[0].write(self.payer);
        accounts[1].write(self.ata);
        accounts[2].write(self.mint);
        accounts[3].write(self.token_program);

        let mut instruction_data = [0u8; CREATE_RENT_PENDING_ATA_DATA_LEN];
        encode_create_rent_pending_ata_data(
            &mut instruction_data,
            self.wallet_owner,
            self.mint.address(),
            self.token_program.address(),
        );

        invoke_signed_with_bounds::<NUM_ACCOUNTS>(
            &InstructionView {
                program_id: self.magic_program.address(),
                accounts: unsafe {
                    from_raw_parts(instruction_accounts.as_ptr() as _, NUM_ACCOUNTS)
                },
                data: &instruction_data,
            },
            unsafe { from_raw_parts(accounts.as_ptr() as _, NUM_ACCOUNTS) },
            signers,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::spl::consts::TOKEN_PROGRAM_ID;

    #[test]
    fn test_encode_create_rent_pending_ata_data() {
        let wallet_owner = Address::new_from_array([1; 32]);
        let mint = Address::new_from_array([2; 32]);
        let token_program = TOKEN_PROGRAM_ID;
        let mut data = [0u8; CREATE_RENT_PENDING_ATA_DATA_LEN];

        encode_create_rent_pending_ata_data(&mut data, &wallet_owner, &mint, &token_program);

        assert_eq!(
            u32::from_le_bytes(data[0..4].try_into().unwrap()),
            CREATE_RENT_PENDING_ATA_DISCRIMINATOR
        );
        assert_eq!(&data[4..36], wallet_owner.as_ref());
        assert_eq!(&data[36..68], mint.as_ref());
        assert_eq!(&data[68..100], token_program.as_ref());
    }

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
