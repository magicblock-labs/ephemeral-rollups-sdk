use solana_program::pubkey::Pubkey;

#[cfg(feature = "anchor")]
pub mod anchor;
pub mod consts;
pub mod cpi;
pub mod delegate_args;
pub mod ephem;
pub mod types;
pub mod utils;

pub use dlp::args::{CallHandlerArgs, Context};
pub use dlp::pda;
pub use dlp::{
    commit_record_seeds_from_delegated_account, commit_state_seeds_from_delegated_account,
    delegate_buffer_seeds_from_delegated_account, delegation_metadata_seeds_from_delegated_account,
    delegation_record_seeds_from_delegated_account, ephemeral_balance_seeds_from_payer,
    fees_vault_seeds, program_config_seeds_from_program_id,
    undelegate_buffer_seeds_from_delegated_account, validator_fees_vault_seeds_from_validator,
};
pub use magicblock_magic_program_api::args::{
    ActionArgs, BaseActionArgs, CommitAndUndelegateArgs, CommitTypeArgs, MagicBaseIntentArgs,
    UndelegateTypeArgs,
};

pub const fn id() -> Pubkey {
    consts::DELEGATION_PROGRAM_ID
}
