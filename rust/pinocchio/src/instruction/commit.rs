use pinocchio::{
    account_info::AccountInfo, cpi::slice_invoke, instruction::Instruction,
    program_error::ProgramError, ProgramResult,
};

use crate::utils::{concate_accounts_with_remaining_accounts, create_schedule_commit_ix};

pub fn commit_accounts(accounts: &[AccountInfo]) -> ProgramResult {
    let [payer, magic_context, magic_program, rest @ ..] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };

    let (ix_data, ix_accounts) = create_schedule_commit_ix(payer, rest, magic_context, false)?;
    let ix = Instruction {
        program_id: magic_program.key(),
        data: &ix_data,
        accounts: &ix_accounts,
    };

    let all_accounts = concate_accounts_with_remaining_accounts(&[payer, magic_context], rest)?;
    //Invoke demands a fixed-sized array so we use slice_invoke
    slice_invoke(&ix, all_accounts)?;
    Ok(())
}
