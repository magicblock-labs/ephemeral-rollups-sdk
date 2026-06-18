//! Minimal Pinocchio counter demonstrating the full Ephemeral Rollups lifecycle:
//! initialize -> increment (base) -> delegate -> increment (ER) -> commit ->
//! commit_and_undelegate.
//!
//! Instruction data is a single tag byte (plus args), except the delegation
//! program's undelegate callback, which arrives prefixed with
//! `EXTERNAL_UNDELEGATE_DISCRIMINATOR`.
//!
//! The counter account stores a single `u64` (little-endian) at offset 0 — no
//! Anchor-style discriminator.
#![no_std]

use ephemeral_rollups_pinocchio::consts::EXTERNAL_UNDELEGATE_DISCRIMINATOR;
use ephemeral_rollups_pinocchio::instruction::{
    commit_accounts, commit_and_undelegate_accounts, delegate_account, undelegate,
};
use ephemeral_rollups_pinocchio::types::DelegateConfig;
use pinocchio::{
    cpi::{Seed, Signer},
    default_allocator,
    error::ProgramError,
    nostd_panic_handler, program_entrypoint,
    sysvars::{rent::Rent, Sysvar},
    AccountView, Address, ProgramResult,
};
use pinocchio_system::instructions::CreateAccount;

program_entrypoint!(process_instruction);
default_allocator!();
nostd_panic_handler!();

/// Seed for the per-payer counter PDA: `["counter", payer]`.
pub const COUNTER_SEED: &[u8] = b"counter";

fn process_instruction(
    program_id: &Address,
    accounts: &[AccountView],
    data: &[u8],
) -> ProgramResult {
    // The delegation program calls back into this program to finish undelegation;
    // its instruction data is prefixed with a fixed 8-byte discriminator.
    if data.len() >= 8 && data[..8] == EXTERNAL_UNDELEGATE_DISCRIMINATOR {
        return process_undelegate(program_id, accounts, &data[8..]);
    }

    let (tag, rest) = data
        .split_first()
        .ok_or(ProgramError::InvalidInstructionData)?;
    match tag {
        0 => initialize(program_id, accounts, rest),
        1 => increment(accounts),
        2 => delegate(accounts, rest),
        3 => commit(accounts, false),
        4 => commit(accounts, true),
        _ => Err(ProgramError::InvalidInstructionData),
    }
}

/// Create the counter PDA on the base layer. `data = [bump]`.
fn initialize(program_id: &Address, accounts: &[AccountView], data: &[u8]) -> ProgramResult {
    let [payer, counter, _system_program] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };
    let bump = *data.first().ok_or(ProgramError::InvalidInstructionData)?;

    let payer_key = payer.address().as_array();
    let bump_arr = [bump];
    let seeds = [
        Seed::from(COUNTER_SEED),
        Seed::from(payer_key.as_ref()),
        Seed::from(&bump_arr),
    ];
    let signer = Signer::from(&seeds);

    let rent = Rent::get()?;
    CreateAccount {
        from: payer,
        to: counter,
        lamports: rent.try_minimum_balance(8)?,
        space: 8,
        owner: program_id,
    }
    .invoke_signed(&[signer])?;

    let mut d = counter.try_borrow_mut()?;
    d[..8].copy_from_slice(&0u64.to_le_bytes());
    Ok(())
}

/// Increment the counter (on base before delegation, on the ER after).
fn increment(accounts: &[AccountView]) -> ProgramResult {
    let [_payer, counter, ..] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };
    if d.len() < 8 {
        return Err(ProgramError::InvalidAccountData);
    }
    let mut d = counter.try_borrow_mut()?;
    let mut v = u64::from_le_bytes(
        d[..8]
            .try_into()
            .map_err(|_| ProgramError::InvalidAccountData)?,
    );
    v = v.wrapping_add(1);
    d[..8].copy_from_slice(&v.to_le_bytes());
    Ok(())
}

/// Delegate the counter PDA. `data = [bump, validator(32)]`.
fn delegate(accounts: &[AccountView], data: &[u8]) -> ProgramResult {
    // The delegation program account must be present so the runtime loads it for
    // the CPI; `delegate_account` itself only takes the first seven accounts.
    let [payer, counter, owner_program, buffer, delegation_record, delegation_metadata, system_program, _delegation_program] =
        accounts
    else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };
    if data.len() < 1 + 32 {
        return Err(ProgramError::InvalidInstructionData);
    }
    let bump = data[0];
    let mut validator = [0u8; 32];
    validator.copy_from_slice(&data[1..33]);

    let payer_key = payer.address().as_array();
    let seeds: [&[u8]; 2] = [COUNTER_SEED, payer_key.as_ref()];
    let config = DelegateConfig {
        commit_frequency_ms: 30_000,
        validator: Some(Address::new_from_array(validator)),
    };
    let accs = [
        payer,
        counter,
        owner_program,
        buffer,
        delegation_record,
        delegation_metadata,
        system_program,
    ];
    delegate_account(&accs, &seeds, bump, config)
}

/// Schedule a commit (and optionally undelegate) of the counter from the ER.
fn commit(accounts: &[AccountView], and_undelegate: bool) -> ProgramResult {
    let [payer, counter, magic_program, magic_context] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };
    let committed = core::slice::from_ref(counter);
    if and_undelegate {
        commit_and_undelegate_accounts(payer, committed, magic_context, magic_program, None, None)
    } else {
        commit_accounts(payer, committed, magic_context, magic_program, None, None)
    }
}

/// Delegation-program callback that re-creates the PDA on the base layer with the
/// committed state. `callback_args` carries the original PDA seeds.
fn process_undelegate(
    program_id: &Address,
    accounts: &[AccountView],
    callback_args: &[u8],
) -> ProgramResult {
    let [counter, buffer, payer, _system_program] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };
    undelegate(counter, program_id, buffer, payer, callback_args)
}
