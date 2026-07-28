use crate::compat::anchor_lang;

pub struct VrfProgram;

impl anchor_lang::Id for VrfProgram {
    fn id() -> anchor_lang::prelude::Pubkey {
        crate::consts::VRF_PROGRAM_ID.to_bytes().into()
    }
}
