mod backward_compat {
    pub use dlp_api::compat::{borsh, Pubkey};

    pub use account_info::AccountInfo;
    pub use solana_account_info_compat as account_info;
    pub use solana_program_compat::instruction::{AccountMeta, Instruction};
    pub use solana_program_error_compat::{ProgramError, ProgramResult};
}

pub use backward_compat::*;

///
/// Modernize
///
pub trait Modernize {
    type Modern;
    fn modernize(&self) -> &Self::Modern;
}

impl Modernize for backward_compat::Pubkey {
    type Modern = solana_address::Address;
    fn modernize(&self) -> &Self::Modern {
        unsafe { &*(self.as_array().as_ptr() as *const Self::Modern) }
    }
}

impl<'info> Modernize for backward_compat::AccountInfo<'info> {
    type Modern = solana_program::account_info::AccountInfo<'info>;
    fn modernize(&self) -> &Self::Modern {
        panic!()
    }
}

impl Modernize for backward_compat::Instruction {
    type Modern = solana_program::instruction::Instruction;
    fn modernize(&self) -> &Self::Modern {
        panic!()
    }
}

impl Modernize for () {
    type Modern = ();
    fn modernize(&self) -> &Self::Modern {
        &()
    }
}

impl Modernize for backward_compat::ProgramError {
    type Modern = solana_program::program_error::ProgramError;
    fn modernize(&self) -> &Self::Modern {
        panic!()
    }
}

impl<T: Modernize, U: Modernize> Modernize for Result<T, U> {
    type Modern = Result<<T as Modernize>::Modern, <U as Modernize>::Modern>;
    fn modernize(&self) -> &Self::Modern {
        panic!()
    }
}

impl<T: Modernize> Modernize for Vec<T> {
    type Modern = Vec<<T as Modernize>::Modern>;
    fn modernize(&self) -> &Self::Modern {
        panic!()
    }
}

///
/// Modernize params of types:
/// - Pubkey
/// - AccountInfo
/// - Anything that impl Modernize
///
#[cfg(feature = "backward-compat")]
#[macro_export]
macro_rules! modernize {
    ($($params:ident),* $(,)?) => {
        $(let $params = $params.modernize();)*
    };
}

#[cfg(not(feature = "backward-compat"))]
#[macro_export]
macro_rules! modernize {
    ($($params:ident),* $(,)?) => {};
}

///
/// Compatize
///
pub trait Compatize {
    type Compat;
    fn compat(self) -> Self::Compat;
}

impl Compatize for solana_address::Address {
    type Compat = backward_compat::Pubkey;
    fn compat(self) -> Self::Compat {
        self.to_bytes().into()
    }
}

impl Compatize for () {
    type Compat = ();
    fn compat(self) -> Self::Compat {
        ()
    }
}

impl Compatize for solana_program::program_error::ProgramError {
    type Compat = backward_compat::ProgramError;
    fn compat(self) -> Self::Compat {
        panic!()
    }
}

//impl Compatize for ProgramError {
//    type Compat = backward_compat::ProgramError;
//    fn compat(self) -> Self::Compat {
//        panic!()
//    }
//}

impl<T: Compatize, U: Compatize> Compatize for Result<T, U> {
    type Compat = Result<<T as Compatize>::Compat, <U as Compatize>::Compat>;
    fn compat(self) -> Self::Compat {
        match self {
            Ok(ok) => Ok(ok.compat()),
            Err(err) => Err(err.compat()),
        }
    }
}

impl Compatize for solana_program::instruction::Instruction {
    type Compat = backward_compat::Instruction;
    fn compat(self) -> Self::Compat {
        panic!()
    }
}
