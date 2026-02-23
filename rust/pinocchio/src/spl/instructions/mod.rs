pub mod create_ephemeral_ata_permission;
pub mod delegate_ephemeral_ata;
pub mod delegate_ephemeral_ata_permission;
pub mod deposit_spl_tokens;
pub mod initialize_ephemeral_ata;
pub mod initialize_global_vault;
pub mod reset_ephemeral_ata_permission;
pub mod undelegate_ephemeral_ata;
pub mod undelegate_ephemeral_ata_permission;
pub mod withdraw_spl_tokens;

pub use create_ephemeral_ata_permission::*;
pub use delegate_ephemeral_ata::*;
pub use delegate_ephemeral_ata_permission::*;
pub use deposit_spl_tokens::*;
pub use initialize_ephemeral_ata::*;
pub use initialize_global_vault::*;
pub use reset_ephemeral_ata_permission::*;
pub use undelegate_ephemeral_ata::*;
pub use undelegate_ephemeral_ata_permission::*;
pub use withdraw_spl_tokens::*;

#[repr(u8)]
#[derive(Debug, PartialEq, Eq, Copy, Clone)]
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
