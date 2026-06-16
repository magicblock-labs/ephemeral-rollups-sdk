pub mod consts;
pub mod instructions;
pub mod rnd;
pub mod types;

pub use crate::compat;

pub const fn id() -> compat::Pubkey {
    consts::VRF_PROGRAM_ID
}
