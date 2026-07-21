#[cfg(feature = "backward-compat")]
extern crate borsh_compat as borsh;

#[cfg(feature = "anchor-support")]
pub mod anchor;
pub mod compat;
pub mod consts;
pub mod instructions;
pub mod pda;
pub mod rnd;
pub mod types;

pub use compat::Pubkey;

pub const fn id() -> Pubkey {
    consts::VRF_PROGRAM_ID
}
