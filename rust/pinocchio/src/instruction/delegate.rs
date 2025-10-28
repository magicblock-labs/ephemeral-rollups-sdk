use pinocchio::{
    account_info::AccountInfo,
    instruction::{Seed, Signer},
    program_error::ProgramError,
    pubkey::find_program_address,
    ProgramResult,
};
use pinocchio_system::instructions::{Assign, CreateAccount};

use crate::consts::DELEGATION_PROGRAM_ID;
use crate::types::DelegateAccountArgs;
use crate::utils::{cpi_delegate, make_seed_buf};
use crate::{consts::BUFFER, types::DelegateConfig, utils::close_pda_acc};

#[allow(clippy::cloned_ref_to_slice_refs)]
pub fn delegate_account(
    accounts: &[&AccountInfo],
    seeds: &[&[u8]],
    bump: u8,
    config: DelegateConfig,
) -> ProgramResult {
    let [payer, pda_acc, owner_program, buffer_acc, delegation_record, delegation_metadata] =
        accounts
    else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };

    if !payer.is_signer() {
        return Err(ProgramError::MissingRequiredSignature);
    }

    // Buffer PDA seeds
    let buffer_seeds: &[&[u8]] = &[BUFFER, pda_acc.key().as_ref()];

    // Bumps
    let (_, buffer_pda_bump) = find_program_address(buffer_seeds, owner_program.key());

    // Buffer signer seeds
    let buffer_bump_slice = [buffer_pda_bump];
    let buffer_seed_binding = [
        Seed::from(BUFFER),
        Seed::from(pda_acc.key().as_ref()),
        Seed::from(&buffer_bump_slice),
    ];
    let buffer_signer_seeds = Signer::from(&buffer_seed_binding);

    // Single data_len and rent lookup
    let data_len = pda_acc.data_len();

    // Create Buffer PDA
    CreateAccount {
        from: payer,
        to: buffer_acc,
        lamports: 0,
        space: data_len as u64,
        owner: owner_program.key(),
    }
    .invoke_signed(&[buffer_signer_seeds])?;

    // Copy delegated PDA -> buffer, then zero delegated PDA
    {
        let pda_ro = pda_acc.try_borrow_data()?;
        let mut buf_data = buffer_acc.try_borrow_mut_data()?;
        buf_data.copy_from_slice(&pda_ro);
    }
    {
        let mut pda_mut = pda_acc.try_borrow_mut_data()?;
        for b in pda_mut.iter_mut().take(data_len) {
            *b = 0;
        }
    }

    // Assign delegated PDA to system if needed, then to delegation program
    let mut seed_buf = make_seed_buf();
    let filled = fill_seeds(&mut seed_buf, seeds, &bump);
    let delegate_signer_seeds = Signer::from(filled);

    let current_owner = unsafe { pda_acc.owner() };
    if current_owner != &pinocchio_system::id() {
        unsafe { pda_acc.assign(&pinocchio_system::id()) };
    }
    let current_owner = unsafe { pda_acc.owner() };
    if current_owner != &DELEGATION_PROGRAM_ID {
        Assign {
            account: pda_acc,
            owner: &DELEGATION_PROGRAM_ID,
        }
        .invoke_signed(&[delegate_signer_seeds.clone()])?;
    }

    // Delegate
    let delegate_args = DelegateAccountArgs {
        commit_frequency_ms: config.commit_frequency_ms,
        seeds,
        validator: config.validator,
    };

    cpi_delegate(
        payer,
        pda_acc,
        owner_program,
        buffer_acc,
        delegation_record,
        delegation_metadata,
        delegate_args,
        delegate_signer_seeds,
    )?;

    // Close buffer PDA back to payer to reclaim lamports
    close_pda_acc(payer, buffer_acc)?;

    Ok(())
}

pub fn fill_seeds<'a>(
    out: &'a mut [Seed<'a>; 16],
    seeds: &[&'a [u8]],
    bump_ref: &'a u8,
) -> &'a [Seed<'a>] {
    assert!(seeds.len() <= 15, "too many seeds (max 15 + bump = 16)");

    let bump_slice: &[u8] = core::slice::from_ref(bump_ref);

    let mut i = 0;
    while i < seeds.len() {
        out[i] = Seed::from(seeds[i]);
        i += 1;
    }
    out[i] = Seed::from(bump_slice);

    &out[..=i]
}
