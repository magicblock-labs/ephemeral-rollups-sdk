use alloc::vec::Vec;
use pinocchio::{
    account_info::AccountInfo,
    cpi::{slice_invoke, MAX_CPI_ACCOUNTS},
    instruction::Instruction,
    program_error::ProgramError,
    ProgramResult,
};

use crate::utils::create_schedule_commit_ix;

pub fn commit_accounts(accounts: &[AccountInfo]) -> ProgramResult {
    let [payer, magic_context, magic_program, rest @ ..] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };

    let (ix_data, ix_accounts) = create_schedule_commit_ix(payer, rest, magic_context, false)?;
    let ix = Instruction {
        program_id: magic_program.key(),
        data: ix_data,
        accounts: ix_accounts,
    };

    let num_accounts = 2 + rest.len(); // payer + magic_context + rest
    if num_accounts > MAX_CPI_ACCOUNTS {
        return Err(ProgramError::InvalidArgument);
    }

    // Build the exact list of account references on the heap to avoid large stack frames
    let mut all_accounts: Vec<&AccountInfo> = Vec::with_capacity(num_accounts);
    all_accounts.push(payer);
    all_accounts.push(magic_context);
    for account in rest.iter() {
        all_accounts.push(account);
    }

    slice_invoke(&ix, &all_accounts)?;
    Ok(())
}
