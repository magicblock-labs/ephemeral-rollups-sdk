use crate::{
    consts::ESPL_TOKEN_PROGRAM_ID,
    solana_compat::solana::{ProgramError, Pubkey},
};
use spl_associated_token_account_interface::address::get_associated_token_address;

/// Internal representation of a token account data.
#[repr(C)]
pub struct EphemeralAta {
    /// The owner of the eata
    pub owner: Pubkey,
    /// The mint associated with this account
    pub mint: Pubkey,
    /// The amount of tokens this account holds.
    pub amount: u64,
}

impl EphemeralAta {
    pub const LEN: usize = 32 + 32 + 8;

    pub fn find_pda(user: &Pubkey, mint: &Pubkey) -> (Pubkey, u8) {
        Pubkey::find_program_address(&[user.as_ref(), mint.as_ref()], &ESPL_TOKEN_PROGRAM_ID)
    }

    pub fn try_from_bytes(bytes: &[u8]) -> Result<Self, ProgramError> {
        if bytes.len() != Self::LEN {
            return Err(ProgramError::InvalidAccountData);
        }
        Ok(Self {
            owner: Pubkey::new_from_array(
                bytes[0..32]
                    .try_into()
                    .map_err(|_| ProgramError::InvalidAccountData)?,
            ),
            mint: Pubkey::new_from_array(
                bytes[32..64]
                    .try_into()
                    .map_err(|_| ProgramError::InvalidAccountData)?,
            ),
            amount: u64::from_le_bytes(
                bytes[64..72]
                    .try_into()
                    .map_err(|_| ProgramError::InvalidAccountData)?,
            ),
        })
    }
}

/// Internal representation of a global vault for a specific mint.
#[repr(C)]
pub struct GlobalVault {
    /// The mint associated with this vault
    pub mint: Pubkey,
}

impl GlobalVault {
    pub const LEN: usize = 32;

    pub fn find_pda(mint: &Pubkey) -> (Pubkey, u8) {
        Pubkey::find_program_address(&[mint.as_ref()], &ESPL_TOKEN_PROGRAM_ID)
    }

    pub fn try_from_bytes(bytes: &[u8]) -> Result<Self, ProgramError> {
        if bytes.len() != Self::LEN {
            return Err(ProgramError::InvalidAccountData);
        }
        Ok(Self {
            mint: Pubkey::new_from_array(
                bytes[0..32]
                    .try_into()
                    .map_err(|_| ProgramError::InvalidAccountData)?,
            ),
        })
    }
}

pub fn find_rent_pda() -> (Pubkey, u8) {
    Pubkey::find_program_address(&[b"rent"], &ESPL_TOKEN_PROGRAM_ID)
}

pub fn find_lamports_pda(payer: &Pubkey, destination: &Pubkey, salt: &[u8; 32]) -> (Pubkey, u8) {
    Pubkey::find_program_address(
        &[b"lamports", payer.as_ref(), destination.as_ref(), salt],
        &ESPL_TOKEN_PROGRAM_ID,
    )
}

pub fn find_vault_ata(mint: &Pubkey, vault: &Pubkey) -> Pubkey {
    get_associated_token_address(vault, mint)
}

pub fn find_shuttle_ephemeral_ata(owner: &Pubkey, mint: &Pubkey, shuttle_id: u32) -> (Pubkey, u8) {
    Pubkey::find_program_address(
        &[owner.as_ref(), mint.as_ref(), &shuttle_id.to_le_bytes()],
        &ESPL_TOKEN_PROGRAM_ID,
    )
}

pub fn find_shuttle_ata(shuttle_ephemeral_ata: &Pubkey, mint: &Pubkey) -> (Pubkey, u8) {
    Pubkey::find_program_address(
        &[shuttle_ephemeral_ata.as_ref(), mint.as_ref()],
        &ESPL_TOKEN_PROGRAM_ID,
    )
}

pub fn find_shuttle_wallet_ata(mint: &Pubkey, shuttle_ephemeral_ata: &Pubkey) -> Pubkey {
    get_associated_token_address(shuttle_ephemeral_ata, mint)
}

pub fn find_transfer_queue(mint: &Pubkey, validator: &Pubkey) -> (Pubkey, u8) {
    Pubkey::find_program_address(
        &[b"queue", mint.as_ref(), validator.as_ref()],
        &ESPL_TOKEN_PROGRAM_ID,
    )
}
