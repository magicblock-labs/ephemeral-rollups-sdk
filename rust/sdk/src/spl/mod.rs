pub mod builders;
pub mod cpi;
pub mod types;

pub use cpi::*;
pub use types::*;

#[repr(u8)]
#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub enum EphemeralSplDiscriminator {
    InitializeEphemeralAta = 0,
    InitializeGlobalVault = 1,
    DepositSplTokens = 2,
    WithdrawSplTokens = 3,
    DelegateEphemeralAta = 4,
    UndelegateEphemeralAta = 5,
    CreateEphemeralAtaPermission = 6,
    DelegateEphemeralAtaPermission = 7,
    UndelegateEphemeralAtaPermission = 8,
    ResetEphemeralAtaPermission = 9,
    CloseEphemeralAta = 10,
}
