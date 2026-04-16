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
    InitializeShuttleEphemeralAta = 11,
    InitializeTransferQueue = 12,
    DelegateShuttleEphemeralAta = 13,
    UndelegateAndCloseShuttleEphemeralAta = 14,
    MergeShuttleIntoAta = 15,
    DepositAndQueueTransfer = 16,
    EnsureTransferQueueCrank = 17,
    DelegateTransferQueue = 19,
    LamportsDelegatedTransfer = 20,
    InitializeRentPda = 23,
    SetupAndDelegateShuttleEphemeralAtaWithMerge = 24,
    DepositAndDelegateShuttleEphemeralAtaWithMergeAndPrivateTransfer = 25,
    WithdrawThroughDelegatedShuttleWithMerge = 26,
    AllocateTransferQueue = 27,
    ProcessPendingTransferQueueRefill = 28,
}
