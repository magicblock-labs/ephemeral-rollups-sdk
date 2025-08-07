use solana_program::pubkey::Pubkey;

#[cfg(feature = "anchor")]
pub mod anchor;
pub mod consts;
pub mod cpi;
pub mod delegate_args;
pub mod ephem;
pub mod types;
pub mod utils;

pub use dlp::args::{CallHandlerArgs, Context};
pub use dlp::pda;
pub use magicblock_core::magic_program::args::{
    ActionArgs, BaseActionArgs, CommitAndUndelegateArgs, CommitTypeArgs, MagicBaseIntentArgs,
    UndelegateTypeArgs,
};

pub const fn id() -> Pubkey {
    consts::DELEGATION_PROGRAM_ID
}
