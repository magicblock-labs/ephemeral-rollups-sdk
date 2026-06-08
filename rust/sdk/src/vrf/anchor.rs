use crate::compat::anchor_lang;

pub use ephemeral_rollups_sdk_attribute_vrf::*;

pub struct VrfProgram;

impl anchor_lang::Id for VrfProgram {
    fn id() -> anchor_lang::prelude::Pubkey {
        crate::vrf::consts::VRF_PROGRAM_ID.to_bytes().into()
    }
}
