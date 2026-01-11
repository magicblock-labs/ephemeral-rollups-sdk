use crate::utils::create_schedule_commit_ix;
use core::mem::MaybeUninit;
use pinocchio::{
    cpi::invoke, error::ProgramError, instruction::InstructionAccount, AccountView, ProgramResult,
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
    if num_accounts > MAX_LOCAL_CPI_ACCOUNTS {
        return Err(ProgramError::InvalidArgument);
    }

    // Build account references array based on actual number of accounts
    // Using a fixed-size array with invoke
    match num_accounts {
        2 => {
            let accs: [&AccountView; 2] = [payer, magic_context];
            invoke(&ix, &accs)?;
        }
        3 => {
            let accs: [&AccountView; 3] = [payer, magic_context, &accounts[0]];
            invoke(&ix, &accs)?;
        }
        4 => {
            let accs: [&AccountView; 4] = [payer, magic_context, &accounts[0], &accounts[1]];
            invoke(&ix, &accs)?;
        }
        5 => {
            let accs: [&AccountView; 5] = [
                payer,
                magic_context,
                &accounts[0],
                &accounts[1],
                &accounts[2],
            ];
            invoke(&ix, &accs)?;
        }
        6 => {
            let accs: [&AccountView; 6] = [
                payer,
                magic_context,
                &accounts[0],
                &accounts[1],
                &accounts[2],
                &accounts[3],
            ];
            invoke(&ix, &accs)?;
        }
        _ => return Err(ProgramError::InvalidArgument),
    }

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
