use pinocchio::{error::ProgramError, AccountView, Address, ProgramResult};

use crate::{types::MembersArgs, utils::cpi_update_permission};

/// Update a permission.
pub fn update_permission(
    accounts: &[&AccountView],
    permission_program: &Address,
    authority_is_signer: bool,
    permissioned_account_is_signer: bool,
    args: MembersArgs,
) -> ProgramResult {
    let [authority, permissioned_account, permission] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };

    if !authority_is_signer && !permissioned_account_is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }

    cpi_update_permission(
        authority,
        permissioned_account,
        permission,
        permission_program,
        authority_is_signer,
        permissioned_account_is_signer,
        args,
    )
}
