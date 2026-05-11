use super::{backward_compat, Compat};

impl Compat for solana_address::Address {
    type Compat = backward_compat::Pubkey;
    fn compat(self) -> Self::Compat {
        self.to_bytes().into()
    }
}

impl Compat for () {
    type Compat = ();
    fn compat(self) -> Self::Compat {
        ()
    }
}

#[cfg(feature = "backward-compat")]
impl Compat for solana_program::program_error::ProgramError {
    type Compat = backward_compat::ProgramError;
    fn compat(self) -> Self::Compat {
        use backward_compat::ProgramError as CompatError;
        use solana_program::program_error::ProgramError as ModernError;

        match self {
            ModernError::Custom(code) => CompatError::Custom(code),
            ModernError::InvalidArgument => CompatError::InvalidArgument,
            ModernError::InvalidInstructionData => CompatError::InvalidInstructionData,
            ModernError::InvalidAccountData => CompatError::InvalidAccountData,
            ModernError::AccountDataTooSmall => CompatError::AccountDataTooSmall,
            ModernError::InsufficientFunds => CompatError::InsufficientFunds,
            ModernError::IncorrectProgramId => CompatError::IncorrectProgramId,
            ModernError::MissingRequiredSignature => CompatError::MissingRequiredSignature,
            ModernError::AccountAlreadyInitialized => CompatError::AccountAlreadyInitialized,
            ModernError::UninitializedAccount => CompatError::UninitializedAccount,
            ModernError::NotEnoughAccountKeys => CompatError::NotEnoughAccountKeys,
            ModernError::AccountBorrowFailed => CompatError::AccountBorrowFailed,
            ModernError::MaxSeedLengthExceeded => CompatError::MaxSeedLengthExceeded,
            ModernError::InvalidSeeds => CompatError::InvalidSeeds,
            ModernError::BorshIoError => CompatError::BorshIoError(String::new()),
            ModernError::AccountNotRentExempt => CompatError::AccountNotRentExempt,
            ModernError::UnsupportedSysvar => CompatError::UnsupportedSysvar,
            ModernError::IllegalOwner => CompatError::IllegalOwner,
            ModernError::MaxAccountsDataAllocationsExceeded => {
                CompatError::MaxAccountsDataAllocationsExceeded
            }
            ModernError::InvalidRealloc => CompatError::InvalidRealloc,
            ModernError::MaxInstructionTraceLengthExceeded => {
                CompatError::MaxInstructionTraceLengthExceeded
            }
            ModernError::BuiltinProgramsMustConsumeComputeUnits => {
                CompatError::BuiltinProgramsMustConsumeComputeUnits
            }
            ModernError::InvalidAccountOwner => CompatError::InvalidAccountOwner,
            ModernError::ArithmeticOverflow => CompatError::ArithmeticOverflow,
            ModernError::Immutable => CompatError::Immutable,
            ModernError::IncorrectAuthority => CompatError::IncorrectAuthority,
        }
    }
}

#[cfg(not(feature = "backward-compat"))]
impl Compat for solana_program::program_error::ProgramError {
    type Compat = backward_compat::ProgramError;
    fn compat(self) -> Self::Compat {
        self
    }
}

impl<T: Compat, U: Compat> Compat for Result<T, U> {
    type Compat = Result<<T as Compat>::Compat, <U as Compat>::Compat>;
    fn compat(self) -> Self::Compat {
        match self {
            Ok(ok) => Ok(ok.compat()),
            Err(err) => Err(err.compat()),
        }
    }
}

impl<T: Compat> Compat for Vec<T> {
    type Compat = Vec<<T as Compat>::Compat>;
    fn compat(self) -> Self::Compat {
        self.into_iter().map(Compat::compat).collect()
    }
}

impl Compat for solana_program::instruction::AccountMeta {
    type Compat = backward_compat::AccountMeta;
    fn compat(self) -> Self::Compat {
        Self::Compat {
            pubkey: self.pubkey.compat(),
            is_signer: self.is_signer,
            is_writable: self.is_writable,
        }
    }
}

impl Compat for solana_program::instruction::Instruction {
    type Compat = backward_compat::Instruction;
    fn compat(self) -> Self::Compat {
        Self::Compat {
            program_id: self.program_id.compat(),
            accounts: self.accounts.compat(),
            data: self.data,
        }
    }
}
