use crate::instruction::commit::commit_accounts_internal;
use pinocchio::{AccountView, ProgramResult};

pub fn commit_and_undelegate_accounts(
    payer: &AccountView,
    accounts: &[AccountView],
    magic_context: &AccountView,
    magic_program: &AccountView,
) -> ProgramResult {
    commit_accounts_internal(payer, accounts, magic_context, magic_program, true)
}
