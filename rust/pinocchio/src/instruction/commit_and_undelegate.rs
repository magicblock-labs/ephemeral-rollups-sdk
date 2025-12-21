use crate::instruction::commit::commit_accounts_internal;
use pinocchio::{account_info::AccountInfo, ProgramResult};

pub fn commit_and_undelegate_accounts(
    payer: &AccountInfo,
    accounts: &[AccountInfo],
    magic_context: &AccountInfo,
    magic_program: &AccountInfo,
) -> ProgramResult {
    commit_accounts_internal(payer, accounts, magic_context, magic_program, true)
}
