pub mod builders;
pub mod cpi;
pub mod types;

pub use cpi::*;
pub use types::*;

pub(crate) mod compat_pda {
    use crate::compat::{self, AsModern, Compat};

    pub fn delegate_buffer_pda_from_delegated_account_and_owner_program(
        delegated_account: &compat::Pubkey,
        owner_program: &compat::Pubkey,
    ) -> compat::Pubkey {
        dlp_api::pda::delegate_buffer_pda_from_delegated_account_and_owner_program(
            delegated_account.as_modern(),
            owner_program.as_modern(),
        )
        .compat()
    }

    pub fn delegation_record_pda_from_delegated_account(
        delegated_account: &compat::Pubkey,
    ) -> compat::Pubkey {
        dlp_api::pda::delegation_record_pda_from_delegated_account(delegated_account.as_modern())
            .compat()
    }

    pub fn delegation_metadata_pda_from_delegated_account(
        delegated_account: &compat::Pubkey,
    ) -> compat::Pubkey {
        dlp_api::pda::delegation_metadata_pda_from_delegated_account(delegated_account.as_modern())
            .compat()
    }
}

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
    SchedulePrivateTransfer = 30,
}
