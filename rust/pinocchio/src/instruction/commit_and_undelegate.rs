use pinocchio::{
    account_info::AccountInfo, cpi::{slice_invoke, MAX_CPI_ACCOUNTS}, instruction::Instruction,
    program_error::ProgramError, ProgramResult,
};
use core::mem::MaybeUninit;

use crate::utils::create_schedule_commit_ix;

pub fn commit_and_undelegate_accounts(accounts: &[AccountInfo]) -> ProgramResult {
    let [magic_context, payer, magic_program, rest @ ..] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };

    let (ix_data, ix_accounts) = create_schedule_commit_ix(payer, rest, magic_context, true)?;
    let ix = Instruction {
        program_id: magic_program.key(),
        data: ix_data,
        accounts: ix_accounts,
    };

    let num_accounts = 2 + rest.len(); // payer + magic_context + rest
    if num_accounts > MAX_CPI_ACCOUNTS {
        return Err(ProgramError::InvalidArgument);
    }
    
    const UNINIT_REF: MaybeUninit<&AccountInfo> = MaybeUninit::<&AccountInfo>::uninit();
    let mut account_refs = [UNINIT_REF; MAX_CPI_ACCOUNTS];
    
    unsafe {
        // SAFETY: num_accounts <= MAX_CPI_ACCOUNTS
        account_refs.get_unchecked_mut(0).write(payer);
        account_refs.get_unchecked_mut(1).write(magic_context);
        
        // Add rest accounts
        for i in 0..rest.len() {
            let account = rest.get_unchecked(i);
            account_refs.get_unchecked_mut(2 + i).write(account);
        }
    }
    
    let all_accounts = unsafe {
        core::slice::from_raw_parts(account_refs.as_ptr() as *const &AccountInfo, num_accounts)
    };
    
    slice_invoke(&ix, all_accounts)?;
    Ok(())
}
