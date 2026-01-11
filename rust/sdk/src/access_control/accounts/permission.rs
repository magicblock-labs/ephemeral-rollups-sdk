use crate::access_control::programs::MAGICBLOCK_PERMISSION_API_ID;
use crate::access_control::types::Member;
use crate::solana_compat::solana::{AccountInfo, Pubkey, PubkeyError};
use borsh::{BorshDeserialize, BorshSerialize};

pub const PERMISSION_SEED: &[u8] = b"permission:";

#[derive(BorshSerialize, BorshDeserialize, Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Permission {
    pub discriminator: u8,
    pub bump: u8,
    #[cfg_attr(
        feature = "serde",
        serde(with = "serde_with::As::<serde_with::DisplayFromStr>")
    )]
    pub permissioned_account: Pubkey,
    pub members: Option<Vec<Member>>,
}

impl Permission {
    /// Prefix values used to generate a PDA for this account.
    ///
    /// Values are positional and appear in the following order:
    ///
    ///   0. `PERMISSION_SEED`
    ///   1. permissioned_account (`Pubkey`)

    pub fn create_pda(permissioned_account: Pubkey, bump: u8) -> Result<Pubkey, PubkeyError> {
        Pubkey::create_program_address(
            &[PERMISSION_SEED, permissioned_account.as_ref(), &[bump]],
            &MAGICBLOCK_PERMISSION_API_ID,
        )
    }

    pub fn find_pda(permissioned_account: &Pubkey) -> (Pubkey, u8) {
        Pubkey::find_program_address(
            &[PERMISSION_SEED, permissioned_account.as_ref()],
            &MAGICBLOCK_PERMISSION_API_ID,
        )
    }

    #[inline(always)]
    pub fn from_bytes(data: &[u8]) -> Result<Self, std::io::Error> {
        let mut data = data;
        Self::deserialize(&mut data)
    }
}

impl<'a> TryFrom<&AccountInfo<'a>> for Permission {
    type Error = std::io::Error;

    fn try_from(account_info: &AccountInfo<'a>) -> Result<Self, Self::Error> {
        let mut data: &[u8] = &(*account_info.data).borrow();
        Self::deserialize(&mut data)
    }
}

#[cfg(feature = "anchor")]
impl anchor_lang::AccountDeserialize for Permission {
    fn try_deserialize_unchecked(buf: &mut &[u8]) -> anchor_lang::Result<Self> {
        Ok(Self::deserialize(buf)?)
    }
}

#[cfg(feature = "anchor")]
impl anchor_lang::AccountSerialize for Permission {}

#[cfg(feature = "anchor")]
impl anchor_lang::Owner for Permission {
    fn owner() -> Pubkey {
        MAGICBLOCK_PERMISSION_API_ID
    }
}
