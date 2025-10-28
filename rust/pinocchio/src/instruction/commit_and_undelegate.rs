use alloc::vec::Vec;
use pinocchio::{
    account_info::AccountInfo,
    cpi::{slice_invoke, MAX_CPI_ACCOUNTS},
    program_error::ProgramError,
    ProgramResult,
};

use crate::utils::create_schedule_commit_ix;

pub fn commit_and_undelegate_accounts(
    payer: &AccountInfo,
    accounts: &[AccountInfo],
    magic_context: &AccountInfo,
    magic_program: &AccountInfo,
) -> ProgramResult {
    let ix = create_schedule_commit_ix(payer, accounts, magic_context, magic_program, true)?;

    let num_accounts = 1 + accounts.len(); // payer + accounts
    if num_accounts > MAX_CPI_ACCOUNTS {
        return Err(ProgramError::InvalidArgument);
    }

    let mut all_accounts: Vec<&AccountInfo> = Vec::with_capacity(accounts.len() + 1);
    all_accounts.push(payer);
    for account in accounts.iter() {
        all_accounts.push(account);
    }

    slice_invoke(&ix, &all_accounts)?;
    Ok(())
}
