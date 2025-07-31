use pinocchio::{
    account_info::AccountInfo, cpi::slice_invoke, instruction::Instruction,
    program_error::ProgramError, ProgramResult,
};

use crate::utils::create_schedule_commit_ix;

pub fn commit_and_undelegate_accounts(accounts: &[AccountInfo]) -> ProgramResult {
    let [magic_context, payer, magic_program, rest @ ..] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };

    let (ix_data, ix_accounts) = create_schedule_commit_ix(payer, rest, magic_context, true);
    let ix = Instruction {
        program_id: magic_program.key(),
        data: &ix_data,
        accounts: &ix_accounts,
    };

    let accounts_for_execute = &accounts[1..];

    let account_refs: &[&AccountInfo] = unsafe {
        std::slice::from_raw_parts(
            accounts_for_execute.as_ptr() as *const &AccountInfo,
            accounts_for_execute.len()
        )
    };

    slice_invoke(&ix, account_refs)?;
    Ok(())
}
