use pinocchio::{
    account_info::AccountInfo, cpi::slice_invoke, instruction::Instruction,
    program_error::ProgramError, ProgramResult,
};

use crate::utils::create_schedule_commit_ix;

pub fn commit_and_undelegate_accounts(accounts: &[AccountInfo]) -> ProgramResult {
    let [payer, magic_context, magic_program, rest @ ..] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };

    let instruction = create_schedule_commit_ix(payer, rest, magic_context, magic_program, true);
    let ix = Instruction {
        program_id: &instruction.program_id,
        data: &instruction.data,
        accounts: &instruction.accounts,
    };

    let mut all_accounts: Vec<&AccountInfo> = vec![payer, magic_context];
    all_accounts.extend(rest.iter());
    //Invoke demands a fixed-sized array so we use slice_invoke
    slice_invoke(&ix, &all_accounts.as_slice())?;
    Ok(())
}
