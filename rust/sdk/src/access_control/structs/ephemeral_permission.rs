use crate::access_control::structs::{Member, PERMISSION_SEED};
use crate::compat::{ProgramError, Pubkey};
use crate::consts::PERMISSION_PROGRAM_ID;

pub struct EphemeralPermission<'a> {
    pub discriminator: u8,
    pub bump: u8,
    pub permissioned_account: Pubkey,
    pub private: bool,
    pub members: &'a [Member],
}

impl<'a> EphemeralPermission<'a> {
    /// Prefix values used to generate a PDA for this account.
    ///
    /// Values are positional and appear in the following order:
    ///
    ///   0. `PERMISSION_SEED`
    ///   1. permissioned_account (`Pubkey`)
    pub const PREFIX: &'static [u8] = PERMISSION_SEED;

    pub fn find_pda(permissioned_account: &Pubkey) -> (Pubkey, u8) {
        Pubkey::find_program_address(
            &[PERMISSION_SEED, permissioned_account.as_ref()],
            &PERMISSION_PROGRAM_ID,
        )
    }

    pub const fn size_of(members: usize) -> usize {
        35 + (1 + members) * Member::SIZE // Account for default member
    }

    pub fn from_bytes(bytes: &'a [u8]) -> Result<Self, ProgramError> {
        if bytes.len() < 35 {
            return Err(ProgramError::InvalidInstructionData);
        }

        let discriminator = bytes[0];
        let bump = bytes[1];
        let permissioned_account = Pubkey::new_from_array(
            bytes[2..34]
                .try_into()
                .map_err(|_| ProgramError::InvalidInstructionData)?,
        );
        let private = bytes[34] == 1;

        if !private {
            return Ok(Self {
                discriminator,
                bump,
                permissioned_account,
                private,
                members: &[],
            });
        }

        let members = bytemuck::try_cast_slice(&bytes[35..])
            .map_err(|_| ProgramError::InvalidInstructionData)?;
        Ok(Self {
            discriminator,
            bump,
            permissioned_account,
            private,
            members,
        })
    }
}
