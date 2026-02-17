pub mod delegate_ephemeral_ata;
pub mod deposit_spl_tokens;
pub mod initialize_ephemeral_ata;
pub mod initialize_global_vault;
pub mod undelegate_ephemeral_ata;
pub mod withdraw_spl_tokens;

#[cfg(feature = "access-control")]
pub mod create_ephemeral_ata_permission;
#[cfg(feature = "access-control")]
pub mod delegate_ephemeral_ata_permission;
#[cfg(feature = "access-control")]
pub mod reset_ephemeral_ata_permission;
#[cfg(feature = "access-control")]
pub mod undelegate_ephemeral_ata_permission;

pub use delegate_ephemeral_ata::*;
pub use deposit_spl_tokens::*;
pub use initialize_ephemeral_ata::*;
pub use initialize_global_vault::*;
pub use undelegate_ephemeral_ata::*;
pub use withdraw_spl_tokens::*;

#[cfg(feature = "access-control")]
pub use create_ephemeral_ata_permission::*;
#[cfg(feature = "access-control")]
pub use delegate_ephemeral_ata_permission::*;
#[cfg(feature = "access-control")]
pub use reset_ephemeral_ata_permission::*;
#[cfg(feature = "access-control")]
pub use undelegate_ephemeral_ata_permission::*;

#[repr(u8)]
pub enum EphemeralSplDiscriminator {
    InitializeGlobalVault = 0,
    InitializeEphemeralAta = 1,
    DepositSplTokens = 2,
    WithdrawSplTokens = 3,
    DelegateEphemeralAta = 4,
    UndelegateEphemeralAta = 5,
    CreateEphemeralAtaPermission = 6,
    DelegateEphemeralAtaPermission = 7,
    UndelegateEphemeralAtaPermission = 8,
    ResetEphemeralAtaPermission = 9,
}
