pub use ephemeral_rollups_sdk_attribute_action::action;
pub use ephemeral_rollups_sdk_attribute_commit::commit;
pub use ephemeral_rollups_sdk_attribute_delegate::delegate;
pub use ephemeral_rollups_sdk_attribute_ephemeral::ephemeral;
pub use ephemeral_rollups_sdk_attribute_ephemeral_accounts::ephemeral_accounts;
#[cfg(feature = "vrf")]
pub use ephemeral_vrf_sdk_vrf_macro::{vrf, vrf_callback};

use crate::compat::anchor_lang;

pub struct DelegationProgram;

impl anchor_lang::Id for DelegationProgram {
    fn id() -> anchor_lang::prelude::Pubkey {
        crate::consts::DELEGATION_PROGRAM_ID.to_bytes().into()
    }
}

pub struct MagicProgram;

impl anchor_lang::Id for MagicProgram {
    fn id() -> anchor_lang::prelude::Pubkey {
        crate::consts::MAGIC_PROGRAM_ID.to_bytes().into()
    }
}

#[cfg(feature = "access-control")]
pub struct PermissionProgram;

#[cfg(feature = "access-control")]
impl anchor_lang::Id for PermissionProgram {
    fn id() -> anchor_lang::prelude::Pubkey {
        crate::consts::PERMISSION_PROGRAM_ID.to_bytes().into()
    }
}

#[cfg(feature = "spl")]
pub struct EsplProgram;

#[cfg(feature = "spl")]
impl anchor_lang::Id for EsplProgram {
    fn id() -> anchor_lang::prelude::Pubkey {
        crate::consts::ESPL_TOKEN_PROGRAM_ID.to_bytes().into()
    }
}
