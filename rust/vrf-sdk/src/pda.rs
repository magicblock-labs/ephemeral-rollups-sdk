use crate::compat::{Compat, Modern, Pubkey};

/// NOTE: Copy/Pasted from delegation-program/src/pda.rs (modify there if needed)
#[macro_export]
macro_rules! delegation_record_seeds_from_delegated_account {
    ($delegated_account: expr) => {
        &[b"delegation", &$delegated_account.as_ref()]
    };
}

#[macro_export]
macro_rules! delegation_metadata_seeds_from_delegated_account {
    ($delegated_account: expr) => {
        &[b"delegation-metadata", &$delegated_account.as_ref()]
    };
}

#[macro_export]
macro_rules! commit_state_seeds_from_delegated_account {
    ($delegated_account: expr) => {
        &[b"state-diff", &$delegated_account.as_ref()]
    };
}

#[macro_export]
macro_rules! commit_record_seeds_from_delegated_account {
    ($delegated_account: expr) => {
        &[b"commit-state-record", &$delegated_account.as_ref()]
    };
}

#[macro_export]
macro_rules! delegate_buffer_seeds_from_delegated_account {
    ($delegated_account: expr) => {
        &[b"buffer", &$delegated_account.as_ref()]
    };
}

#[macro_export]
macro_rules! undelegate_buffer_seeds_from_delegated_account {
    ($delegated_account: expr) => {
        &[b"undelegate-buffer", &$delegated_account.as_ref()]
    };
}

#[macro_export]
macro_rules! fees_vault_seeds {
    () => {
        &[b"fees-vault"]
    };
}

#[macro_export]
macro_rules! validator_fees_vault_seeds_from_validator {
    ($validator: expr) => {
        &[b"v-fees-vault", &$validator.as_ref()]
    };
}

#[macro_export]
macro_rules! program_config_seeds_from_program_id {
    ($program_id: expr) => {
        &[b"p-conf", &$program_id.as_ref()]
    };
}

#[macro_export]
macro_rules! ephemeral_balance_seeds_from_payer {
    ($payer: expr, $index: expr) => {
        &[b"balance", &$payer.as_ref(), &[$index]]
    };
}

pub fn delegation_record_pda_from_delegated_account(delegated_account: &Pubkey) -> Pubkey {
    let delegated_account = (*delegated_account).modern();
    crate::compat::latest::Pubkey::find_program_address(
        delegation_record_seeds_from_delegated_account!(delegated_account),
        &crate::id().modern(),
    )
    .0
    .compat()
}

pub fn delegation_metadata_pda_from_delegated_account(delegated_account: &Pubkey) -> Pubkey {
    let delegated_account = (*delegated_account).modern();
    crate::compat::latest::Pubkey::find_program_address(
        delegation_metadata_seeds_from_delegated_account!(delegated_account),
        &crate::id().modern(),
    )
    .0
    .compat()
}

pub fn commit_state_pda_from_delegated_account(delegated_account: &Pubkey) -> Pubkey {
    let delegated_account = (*delegated_account).modern();
    crate::compat::latest::Pubkey::find_program_address(
        commit_state_seeds_from_delegated_account!(delegated_account),
        &crate::id().modern(),
    )
    .0
    .compat()
}

pub fn commit_record_pda_from_delegated_account(delegated_account: &Pubkey) -> Pubkey {
    let delegated_account = (*delegated_account).modern();
    crate::compat::latest::Pubkey::find_program_address(
        commit_record_seeds_from_delegated_account!(delegated_account),
        &crate::id().modern(),
    )
    .0
    .compat()
}

pub fn delegate_buffer_pda_from_delegated_account_and_owner_program(
    delegated_account: &Pubkey,
    owner_program: &Pubkey,
) -> Pubkey {
    let delegated_account = (*delegated_account).modern();
    let owner_program = (*owner_program).modern();
    crate::compat::latest::Pubkey::find_program_address(
        delegate_buffer_seeds_from_delegated_account!(delegated_account),
        &owner_program,
    )
    .0
    .compat()
}

pub fn undelegate_buffer_pda_from_delegated_account(delegated_account: &Pubkey) -> Pubkey {
    let delegated_account = (*delegated_account).modern();
    crate::compat::latest::Pubkey::find_program_address(
        undelegate_buffer_seeds_from_delegated_account!(delegated_account),
        &crate::id().modern(),
    )
    .0
    .compat()
}

pub fn fees_vault_pda() -> Pubkey {
    crate::compat::latest::Pubkey::find_program_address(fees_vault_seeds!(), &crate::id().modern())
        .0
        .compat()
}

pub fn validator_fees_vault_pda_from_validator(validator: &Pubkey) -> Pubkey {
    let validator = (*validator).modern();
    crate::compat::latest::Pubkey::find_program_address(
        validator_fees_vault_seeds_from_validator!(validator),
        &crate::id().modern(),
    )
    .0
    .compat()
}

pub fn program_config_from_program_id(program_id: &Pubkey) -> Pubkey {
    let program_id = (*program_id).modern();
    crate::compat::latest::Pubkey::find_program_address(
        program_config_seeds_from_program_id!(program_id),
        &crate::id().modern(),
    )
    .0
    .compat()
}

pub fn ephemeral_balance_pda_from_payer(payer: &Pubkey, index: u8) -> Pubkey {
    let payer = (*payer).modern();
    crate::compat::latest::Pubkey::find_program_address(
        ephemeral_balance_seeds_from_payer!(payer, index),
        &crate::id().modern(),
    )
    .0
    .compat()
}
