use pinocchio::{error::ProgramError, AccountView, Address, ProgramResult};

use crate::{types::MembersArgs, utils::cpi_create_permission};

/// Create a new permission for a delegated account.
pub fn create_permission(
    accounts: &[&AccountView],
    permission_program: &Address,
    args: MembersArgs,
) -> ProgramResult {
    let [permissioned_account, permission, payer, system_program] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };

    if !permissioned_account.is_signer() {
        return Err(ProgramError::MissingRequiredSignature);
    }

    cpi_create_permission(
        permissioned_account,
        permission,
        payer,
        system_program,
        permission_program,
        args,
    )
}
