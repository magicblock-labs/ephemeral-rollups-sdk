use crate::compat;

#[cfg(feature = "anchor-support")]
pub mod anchor;
pub mod consts;
pub mod instructions;
pub mod rnd;
pub mod types;

pub const fn id() -> compat::Pubkey {
    consts::VRF_PROGRAM_ID
}
