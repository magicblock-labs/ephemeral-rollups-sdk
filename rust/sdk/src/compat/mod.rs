#![allow(unused_imports)]
#![allow(dead_code)]
#![allow(clippy::empty_line_after_doc_comments)]

///
/// compat is a boundary layer for the public API.
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
mod as_modern;
mod compatize;
mod modern;

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

///
/// Owned modernization for values that need field-by-field conversion.
///
pub trait Modern {
    type Modern;
    fn modern(self) -> Self::Modern;
}

///
/// Owned compatibility conversion.
///
pub trait Compat {
    type Compat;
    fn compat(self) -> Self::Compat;
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
