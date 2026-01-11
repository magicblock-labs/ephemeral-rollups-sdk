use crate::solana_compat::solana::ProgramError;
use num_derive::FromPrimitive;
use thiserror::Error;

#[derive(Clone, Debug, Eq, Error, FromPrimitive, PartialEq)]
pub enum MagicblockPermissionApiError {
    /// 0 - Invalid System Program
    #[error("Invalid System Program")]
    InvalidSystemProgram = 0x0,
    /// 1 - Error deserializing account
    #[error("Error deserializing account")]
    DeserializationError = 0x1,
    /// 2 - Error serializing account
    #[error("Error serializing account")]
    SerializationError = 0x2,
    /// 3 - Invalid Group Size
    #[error("Invalid Group Size")]
    InvalidGroupSize = 0x3,
    /// 4 - Invalid Owner
    #[error("Invalid Owner")]
    InvalidOwner = 0x4,
}

impl From<MagicblockPermissionApiError> for ProgramError {
    fn from(e: MagicblockPermissionApiError) -> Self {
        ProgramError::Custom(e as u32)
    }
}
