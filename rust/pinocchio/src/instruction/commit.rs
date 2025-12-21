use crate::utils::create_schedule_commit_ix;
use core::mem::MaybeUninit;
use pinocchio::instruction::AccountMeta;
use pinocchio::{
    account_info::AccountInfo, cpi::slice_invoke, program_error::ProgramError, ProgramResult,
};

const MAX_LOCAL_CPI_ACCOUNTS: usize = 16;

pub(crate) fn commit_accounts_internal(
    payer: &AccountInfo,
    accounts: &[AccountInfo],
    magic_context: &AccountInfo,
    magic_program: &AccountInfo,
    allow_undelegation: bool,
) -> ProgramResult {
    let mut metas: [MaybeUninit<AccountMeta>; MAX_LOCAL_CPI_ACCOUNTS] = unsafe {
        MaybeUninit::<[MaybeUninit<AccountMeta>; MAX_LOCAL_CPI_ACCOUNTS]>::uninit().assume_init()
    };

    let ix = create_schedule_commit_ix(
        payer,
        accounts,
        magic_context,
        magic_program,
        allow_undelegation,
        &mut metas,
    )?;

    let num_accounts = ix.accounts.len();
    if num_accounts > MAX_LOCAL_CPI_ACCOUNTS {
        return Err(ProgramError::InvalidArgument);
    }

    let mut all_accounts: [&AccountInfo; MAX_LOCAL_CPI_ACCOUNTS] = [payer; MAX_LOCAL_CPI_ACCOUNTS];

    all_accounts[0] = payer;
    all_accounts[1] = magic_context;

    let mut i = 0usize;
    while i < accounts.len() {
        all_accounts[2 + i] = &accounts[i];
        i += 1;
    }

    slice_invoke(&ix, &all_accounts[..num_accounts])?;
    Ok(())
}

pub fn commit_accounts(
    payer: &AccountInfo,
    accounts: &[AccountInfo],
    magic_context: &AccountInfo,
    magic_program: &AccountInfo,
) -> ProgramResult {
    commit_accounts_internal(payer, accounts, magic_context, magic_program, false)
}
