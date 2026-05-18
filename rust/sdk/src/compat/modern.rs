use super::{backward_compat, AsModern, Modern};

impl<T: AsModern + ?Sized> Modern for &T
where
    T::Modern: Clone,
{
    type Modern = T::Modern;
    fn modern(self) -> Self::Modern {
        self.as_modern().clone()
    }
}

impl Modern for backward_compat::Pubkey {
    type Modern = solana_address::Address;
    fn modern(self) -> Self::Modern {
        *self.as_modern()
    }
}

impl<'info> Modern for backward_compat::AccountInfo<'info> {
    type Modern = solana_program::account_info::AccountInfo<'info>;
    fn modern(self) -> Self::Modern {
        self.as_modern().clone()
    }
}

impl Modern for backward_compat::AccountMeta {
    type Modern = solana_program::instruction::AccountMeta;
    fn modern(self) -> Self::Modern {
        Self::Modern {
            pubkey: self.pubkey.modern(),
            is_signer: self.is_signer,
            is_writable: self.is_writable,
        }
    }
}

impl Modern for backward_compat::Instruction {
    type Modern = solana_program::instruction::Instruction;
    fn modern(self) -> Self::Modern {
        Self::Modern {
            program_id: self.program_id.modern(),
            accounts: self.accounts.modern(),
            data: self.data,
        }
    }
}

impl Modern for () {
    type Modern = ();
    fn modern(self) -> Self::Modern {
        self
    }
}

#[cfg(feature = "backward-compat")]
impl Modern for backward_compat::ProgramError {
    type Modern = solana_program::program_error::ProgramError;
    fn modern(self) -> Self::Modern {
        use backward_compat::ProgramError as CompatError;
        use solana_program::program_error::ProgramError as ModernError;

        match self {
            CompatError::Custom(code) => ModernError::Custom(code),
            CompatError::InvalidArgument => ModernError::InvalidArgument,
            CompatError::InvalidInstructionData => ModernError::InvalidInstructionData,
            CompatError::InvalidAccountData => ModernError::InvalidAccountData,
            CompatError::AccountDataTooSmall => ModernError::AccountDataTooSmall,
            CompatError::InsufficientFunds => ModernError::InsufficientFunds,
            CompatError::IncorrectProgramId => ModernError::IncorrectProgramId,
            CompatError::MissingRequiredSignature => ModernError::MissingRequiredSignature,
            CompatError::AccountAlreadyInitialized => ModernError::AccountAlreadyInitialized,
            CompatError::UninitializedAccount => ModernError::UninitializedAccount,
            CompatError::NotEnoughAccountKeys => ModernError::NotEnoughAccountKeys,
            CompatError::AccountBorrowFailed => ModernError::AccountBorrowFailed,
            CompatError::MaxSeedLengthExceeded => ModernError::MaxSeedLengthExceeded,
            CompatError::InvalidSeeds => ModernError::InvalidSeeds,
            CompatError::BorshIoError(_) => ModernError::BorshIoError,
            CompatError::AccountNotRentExempt => ModernError::AccountNotRentExempt,
            CompatError::UnsupportedSysvar => ModernError::UnsupportedSysvar,
            CompatError::IllegalOwner => ModernError::IllegalOwner,
            CompatError::MaxAccountsDataAllocationsExceeded => {
                ModernError::MaxAccountsDataAllocationsExceeded
            }
            CompatError::InvalidRealloc => ModernError::InvalidRealloc,
            CompatError::MaxInstructionTraceLengthExceeded => {
                ModernError::MaxInstructionTraceLengthExceeded
            }
            CompatError::BuiltinProgramsMustConsumeComputeUnits => {
                ModernError::BuiltinProgramsMustConsumeComputeUnits
            }
            CompatError::InvalidAccountOwner => ModernError::InvalidAccountOwner,
            CompatError::ArithmeticOverflow => ModernError::ArithmeticOverflow,
            CompatError::Immutable => ModernError::Immutable,
            CompatError::IncorrectAuthority => ModernError::IncorrectAuthority,
        }
    }
}

#[cfg(not(feature = "backward-compat"))]
impl Modern for backward_compat::ProgramError {
    type Modern = solana_program::program_error::ProgramError;
    fn modern(self) -> Self::Modern {
        self
    }
}

impl<T: Modern, U: Modern> Modern for Result<T, U> {
    type Modern = Result<<T as Modern>::Modern, <U as Modern>::Modern>;
    fn modern(self) -> Self::Modern {
        match self {
            Ok(ok) => Ok(ok.modern()),
            Err(err) => Err(err.modern()),
        }
    }
}

impl<T: Modern> Modern for Vec<T> {
    type Modern = Vec<<T as Modern>::Modern>;
    fn modern(self) -> Self::Modern {
        self.into_iter().map(Modern::modern).collect()
    }
}

impl<T: Modern, const N: usize> Modern for [T; N] {
    type Modern = [<T as Modern>::Modern; N];
    fn modern(self) -> Self::Modern {
        self.map(Modern::modern)
    }
}
