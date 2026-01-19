use pinocchio::{error::ProgramError, AccountView, Address, ProgramResult};
use pinocchio::cpi::Signer;

use crate::utils::cpi_close_permission;

/// Close a permission and recover rent.
pub fn close_permission(
    accounts: &[&AccountView],
    permission_program: &Address,
    authority_is_signer: bool,
    permissioned_account_is_signer: bool,
    signer_seeds: Option<Signer<'_, '_>>,
) -> ProgramResult {
    let [payer, authority, permissioned_account, permission] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };

    if !authority_is_signer && !permissioned_account_is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }

    cpi_close_permission(
        payer,
        authority,
        permissioned_account,
        permission,
        permission_program,
        authority_is_signer,
        permissioned_account_is_signer,
        signer_seeds,
    )
}
