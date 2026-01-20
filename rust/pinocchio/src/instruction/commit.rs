use crate::utils::create_schedule_commit_ix;
use core::mem::MaybeUninit;
use pinocchio::{
    cpi::invoke_signed_with_bounds, error::ProgramError, instruction::InstructionAccount,
    AccountView, ProgramResult,
};

const MAX_LOCAL_CPI_ACCOUNTS: usize = 16;

pub(crate) fn commit_accounts_internal(
    payer: &AccountView,
    accounts: &[AccountView],
    magic_context: &AccountView,
    magic_program: &AccountView,
    allow_undelegation: bool,
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
        allow_undelegation,
        &mut metas,
    )?;

    let num_accounts = ix.accounts.len();
    if num_accounts > MAX_LOCAL_CPI_ACCOUNTS || accounts.len() > MAX_LOCAL_CPI_ACCOUNTS - 2 {
        return Err(ProgramError::InvalidArgument);
    }

    let mut all_accounts: [&AccountView; MAX_LOCAL_CPI_ACCOUNTS] = [payer; MAX_LOCAL_CPI_ACCOUNTS];
    all_accounts[0] = payer;
    all_accounts[1] = magic_context;

    let mut i = 0;
    while i < accounts.len() {
        all_accounts[2 + i] = &accounts[i];
        i += 1;
    }

    invoke_signed_with_bounds::<MAX_LOCAL_CPI_ACCOUNTS>(&ix, &all_accounts[..num_accounts], &[])?;

    Ok(())
}

pub fn commit_accounts(
    payer: &AccountView,
    accounts: &[AccountView],
    magic_context: &AccountView,
    magic_program: &AccountView,
) -> ProgramResult {
    commit_accounts_internal(payer, accounts, magic_context, magic_program, false)
}
