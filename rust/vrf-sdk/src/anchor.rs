use crate::compat::anchor_lang;

pub use ephemeral_vrf_sdk_vrf_macro::*;

pub struct VrfProgram;

impl anchor_lang::Id for VrfProgram {
    fn id() -> anchor_lang::prelude::Pubkey {
        crate::consts::VRF_PROGRAM_ID.to_bytes().into()
    }
}
