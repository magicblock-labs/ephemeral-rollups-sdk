pub use ephemeral_rollups_sdk_attribute_action::action;
pub use ephemeral_rollups_sdk_attribute_commit::commit;
pub use ephemeral_rollups_sdk_attribute_delegate::delegate;
pub use ephemeral_rollups_sdk_attribute_ephemeral::ephemeral;
pub use ephemeral_rollups_sdk_attribute_ephemeral_accounts::ephemeral_accounts;
extern crate anchor_lang;

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
