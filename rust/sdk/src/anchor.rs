#[cfg(feature = "anchor")]
pub use ephemeral_rollups_sdk_attribute_ephemeral::ephemeral;

#[cfg(feature = "anchor")]
pub use ephemeral_rollups_sdk_attribute_commit::commit;

#[cfg(feature = "anchor")]
pub use ephemeral_rollups_sdk_attribute_delegate::delegate;

#[cfg(feature = "anchor")]
use solana_program::pubkey::Pubkey;

#[cfg(feature = "anchor")]
extern crate anchor_lang;

#[cfg(feature = "anchor")]
pub struct DelegationProgram;

#[cfg(feature = "anchor")]
impl anchor_lang::Id for DelegationProgram {
    fn id() -> Pubkey {
        crate::consts::DELEGATION_PROGRAM_ID
    }
}

#[cfg(feature = "anchor")]
pub struct MagicProgram;

#[cfg(feature = "anchor")]
impl anchor_lang::Id for MagicProgram {
    fn id() -> Pubkey {
        crate::consts::MAGIC_PROGRAM_ID
    }
}
