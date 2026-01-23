use pinocchio::Address;
use solana_address::error::AddressError;

use crate::acl::consts::{PERMISSION, PERMISSION_PROGRAM_ID};

pub fn permission_pda_from_permissioned_account(permissioned_account: &Address) -> Address {
    let (pda, _bump) = crate::pda::find_program_address(
        &[PERMISSION, permissioned_account.as_ref()],
        &PERMISSION_PROGRAM_ID,
    );
    pda
}

pub fn permission_pda_from_permissioned_account_with_bump(
    permissioned_account: &Address,
    bump: u8,
) -> Result<Address, AddressError> {
    Address::create_program_address(
        &[PERMISSION, permissioned_account.as_ref(), &[bump]],
        &PERMISSION_PROGRAM_ID,
    )
}
