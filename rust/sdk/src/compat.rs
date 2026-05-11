#![allow(unused_imports)]
#![allow(dead_code)]
#![allow(clippy::empty_line_after_doc_comments)]

///
/// compat.rs is a boundary layer for the public API
/// ================================================
///
/// It lets sdk expose either the legacy compatibility types or the current
/// Solana 3.0 types at the API surface, while keeping the implementation itself
/// always on Solana 3.0.
///
/// In practice, compat::{Pubkey, borsh, ...} is used only for public function
/// parameters and return types. As soon as execution enters a function body, inputs
/// are normalized to the Solana 3.0 types, and the internal logic runs entirely on
/// Solana 3.0. If a value needs to cross back out through the public API, it is
/// converted back at the boundary.
///

#[cfg(feature = "backward-compat")]
mod backward_compat {
    pub use dlp_api::compat::{borsh, Pubkey};

    pub use account_info::AccountInfo;
    pub use solana_account_info_compat as account_info;
    pub use solana_program_compat::instruction::{AccountMeta, Instruction};
    pub use solana_program_error_compat::{ProgramError, ProgramResult};
}

#[cfg(not(feature = "backward-compat"))]
mod backward_compat {
    pub use dlp_api::compat::{borsh, Pubkey};

    pub use account_info::AccountInfo;
    pub use solana_program::account_info;
    pub use solana_program::entrypoint_deprecated::ProgramResult;
    pub use solana_program::instruction::{AccountMeta, Instruction};
    pub use solana_program::program_error::ProgramError;
}

pub use backward_compat::*;

///
/// Borrowed modernization for layout-compatible values.
///
pub trait AsModern {
    type Modern: ?Sized;
    fn as_modern(&self) -> &Self::Modern;
}

#[cfg(feature = "backward-compat")]
impl AsModern for backward_compat::Pubkey {
    type Modern = solana_address::Address;
    fn as_modern(&self) -> &Self::Modern {
        unsafe { &*(self.as_array().as_ptr() as *const Self::Modern) }
    }
}

#[cfg(not(feature = "backward-compat"))]
impl AsModern for backward_compat::Pubkey {
    type Modern = solana_address::Address;
    fn as_modern(&self) -> &Self::Modern {
        self
    }
}

impl<'info> AsModern for backward_compat::AccountInfo<'info> {
    type Modern = solana_program::account_info::AccountInfo<'info>;
    fn as_modern(&self) -> &Self::Modern {
        const {
            assert!(
                core::mem::size_of::<backward_compat::AccountInfo<'static>>()
                    == core::mem::size_of::<solana_program::account_info::AccountInfo<'static>>()
            );
            assert!(
                core::mem::align_of::<backward_compat::AccountInfo<'static>>()
                    == core::mem::align_of::<solana_program::account_info::AccountInfo<'static>>()
            );
        }

        unsafe { &*(self as *const Self as *const Self::Modern) }
    }
}

impl AsModern for () {
    type Modern = ();
    fn as_modern(&self) -> &Self::Modern {
        self
    }
}

///
/// Owned modernization for values that need field-by-field conversion.
///
pub trait Modern {
    type Modern;
    fn modern(self) -> Self::Modern;
}

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

///
/// Borrow params as modern references:
/// - Pubkey
/// - AccountInfo
/// - Anything that impl AsModern
///
#[macro_export]
macro_rules! modernize {
    ($($params:ident),* $(,)?) => {
        $(let $params = $params.as_modern();)*
    };
}

///
/// Owned compatibility conversion.
///
pub trait Compat {
    type Compat;
    fn compat(self) -> Self::Compat;
}

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
