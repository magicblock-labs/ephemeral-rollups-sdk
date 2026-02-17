use crate::spl::ESPL_TOKEN_PROGRAM_ID;
use pinocchio::{error::ProgramError, Address};

/// Represents a single account in an instruction.
#[derive(Debug, Clone)]
pub struct InstructionAccount {
    pub address: Address,
    pub is_writable: bool,
    pub is_signer: bool,
}

/// Internal representation of a token account data.
#[repr(C)]
pub struct EphemeralAta {
    /// The owner of the eata
    pub owner: Address,
    /// The mint associated with this account
    pub mint: Address,
    /// The amount of tokens this account holds.
    pub amount: u64,
}

impl EphemeralAta {
    pub const LEN: usize = 32 + 32 + 8;

    pub fn find_pda(user: &Address, mint: &Address) -> (Address, u8) {
        Address::find_program_address(&[user.as_ref(), mint.as_ref()], &ESPL_TOKEN_PROGRAM_ID)
    }

    pub fn try_from_bytes(bytes: &[u8]) -> Result<Self, ProgramError> {
        if bytes.len() != Self::LEN {
            return Err(ProgramError::InvalidAccountData);
        }
        Ok(Self {
            owner: Address::new_from_array(
                bytes[0..32]
                    .try_into()
                    .map_err(|_| ProgramError::InvalidAccountData)?,
            ),
            mint: Address::new_from_array(
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
    pub mint: Address,
}

impl GlobalVault {
    pub const LEN: usize = 32;

    pub fn find_pda(mint: &Address) -> (Address, u8) {
        Address::find_program_address(&[mint.as_ref()], &ESPL_TOKEN_PROGRAM_ID)
    }

    pub fn try_from_bytes(bytes: &[u8]) -> Result<Self, ProgramError> {
        if bytes.len() != Self::LEN {
            return Err(ProgramError::InvalidAccountData);
        }
        Ok(Self {
            mint: Address::new_from_array(
                bytes[0..32]
                    .try_into()
                    .map_err(|_| ProgramError::InvalidAccountData)?,
            ),
        })
    }
}
