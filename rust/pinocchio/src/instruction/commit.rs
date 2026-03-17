use crate::utils::create_schedule_commit_ix;
use core::mem::MaybeUninit;
use pinocchio::{
    cpi::{invoke_signed_with_bounds, Signer},
    error::ProgramError,
    instruction::InstructionAccount,
    AccountView, ProgramResult,
};

const MAX_LOCAL_CPI_ACCOUNTS: usize = 16;

pub(crate) fn commit_accounts_internal(
    payer: &AccountView,
    accounts: &[AccountView],
    magic_context: &AccountView,
    magic_program: &AccountView,
    magic_fee_vault: Option<&AccountView>,
    allow_undelegation: bool,
    signer_seeds: Option<&[Signer<'_, '_>]>,
) -> ProgramResult {
    let mut metas: [MaybeUninit<InstructionAccount>; MAX_LOCAL_CPI_ACCOUNTS] = unsafe {
        MaybeUninit::<[MaybeUninit<InstructionAccount>; MAX_LOCAL_CPI_ACCOUNTS]>::uninit()
            .assume_init()
    };

    let ix = create_schedule_commit_ix(
        payer,
        accounts,
        magic_context,
        magic_program,
        magic_fee_vault,
        allow_undelegation,
        &mut metas,
    )?;

    let num_prefix_accounts = if magic_fee_vault.is_some() { 3 } else { 2 };
    if ix.accounts.len() > MAX_LOCAL_CPI_ACCOUNTS
        || accounts.len() > MAX_LOCAL_CPI_ACCOUNTS - num_prefix_accounts
    {
        return Err(ProgramError::InvalidArgument);
    }

    let mut all_accounts: [&AccountView; MAX_LOCAL_CPI_ACCOUNTS] = [payer; MAX_LOCAL_CPI_ACCOUNTS];
    all_accounts[0] = payer;
    all_accounts[1] = magic_context;
    if let Some(vault) = magic_fee_vault {
        all_accounts[2] = vault;
    }

    let mut i = 0;
    while i < accounts.len() {
        all_accounts[num_prefix_accounts + i] = &accounts[i];
        i += 1;
    }

    invoke_signed_with_bounds::<MAX_LOCAL_CPI_ACCOUNTS>(
        &ix,
        &all_accounts[..ix.accounts.len()],
        signer_seeds.unwrap_or(&[]),
    )?;

    Ok(())
}

pub fn commit_accounts(
    payer: &AccountView,
    accounts: &[AccountView],
    magic_context: &AccountView,
    magic_program: &AccountView,
    magic_fee_vault: Option<&AccountView>,
    signer_seeds: Option<&[Signer<'_, '_>]>,
) -> ProgramResult {
    commit_accounts_internal(
        payer,
        accounts,
        magic_context,
        magic_program,
        magic_fee_vault,
        false,
        signer_seeds,
    )
}
