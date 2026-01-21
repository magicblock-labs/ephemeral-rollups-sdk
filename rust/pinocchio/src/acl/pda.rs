use pinocchio::Address;

use crate::acl::consts::{PERMISSION, PERMISSION_PROGRAM_ID};

pub fn permission_pda_from_permissioned_account(permissioned_account: &Address) -> Address {
    let (pda, _bump) = crate::pda::find_program_address(
        &[PERMISSION, permissioned_account.as_ref()],
        &PERMISSION_PROGRAM_ID,
    );
    pda
}
