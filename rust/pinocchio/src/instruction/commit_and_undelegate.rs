use crate::instruction::commit::commit_accounts_internal;
use pinocchio::{cpi::Signer, AccountView, ProgramResult};

pub fn commit_and_undelegate_accounts(
    payer: &AccountView,
    accounts: &[AccountView],
    magic_context: &AccountView,
    magic_program: &AccountView,
    magic_fee_vault: Option<&AccountView>,
    signer_seeds: Option<&[Signer<'_, '_>]>,
) -> ProgramResult {
    commit_accounts_internal(
        payer,
        accounts,
        magic_context,
        magic_program,
        magic_fee_vault,
        true,
        signer_seeds,
    )
}
