use pinocchio::program_error::ProgramError;

#[derive(Clone, PartialEq)]
pub enum MyProgramError {
    // overflow error
    WriteOverflow,
    // invalid instruction data
    InvalidInstructionData,
    // pda mismatch
    PdaMismatch,
    // Invalid Owner
    InvalidOwner,
    // Not a system account
    InvalidAccount,
    //Unable to Deserialize
    DeserializationFailed,
    //Unable to Serialize
    SerializationFailed,
    FailedRealloc,
    InvalidIxData,
}

impl From<MyProgramError> for ProgramError {
    fn from(e: MyProgramError) -> Self {
        Self::Custom(e as u32)
    }
}
