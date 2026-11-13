#[cfg(feature = "anchor")]
pub mod anchor;
pub mod consts;
pub mod instructions;
pub mod rnd;
pub mod types;

pub use solana_program::pubkey::Pubkey;

pub const fn id() -> Pubkey {
    consts::VRF_PROGRAM_ID
}
