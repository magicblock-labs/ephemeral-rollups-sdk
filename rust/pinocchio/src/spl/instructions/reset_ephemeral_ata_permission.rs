use core::mem::MaybeUninit;

use crate::spl::consts::ESPL_TOKEN_PROGRAM_ID;
use crate::spl::EphemeralSplDiscriminator;
use pinocchio::cpi::{invoke, invoke_signed, Signer, MAX_CPI_ACCOUNTS};
use pinocchio::instruction::{InstructionAccount, InstructionView};
use pinocchio::{AccountView, ProgramResult};

/// Reset an ephemeral ATA permission.
///
/// For details on the flag byte, see the [MemberFlags](`crate::access_control::structs::Member`) struct.
pub fn reset_ephemeral_ata_permission(
    eata: AccountView,
    permission: AccountView,
    owner: AccountView,
    permission_program: AccountView,
    bump: u8,
    flag_byte: u8,
    signer_seeds: Option<Signer<'_, '_>>,
) -> ProgramResult {
    let mut account_metas =
        [const { MaybeUninit::<InstructionAccount>::uninit() }; MAX_CPI_ACCOUNTS];
    let num_accounts = 4;

    unsafe {
        account_metas
            .get_unchecked_mut(0)
            .write(InstructionAccount::writable(eata.address()));
        account_metas
            .get_unchecked_mut(1)
            .write(InstructionAccount::writable(permission.address()));
        account_metas
            .get_unchecked_mut(2)
            .write(InstructionAccount::readonly_signer(owner.address()));
        account_metas
            .get_unchecked_mut(3)
            .write(InstructionAccount::readonly(permission_program.address()));
    }

    let acc_infos: [&AccountView; 4] = [&eata, &permission, &owner, &permission_program];

    let data: [u8; 3] = [
        EphemeralSplDiscriminator::ResetEphemeralAtaPermission as u8,
        bump,
        flag_byte,
    ];

    let ix = InstructionView {
        program_id: &ESPL_TOKEN_PROGRAM_ID,
        accounts: unsafe {
            core::slice::from_raw_parts(
                account_metas.as_ptr() as *const InstructionAccount,
                num_accounts,
            )
        },
        data: &data,
    };

    if let Some(seeds) = signer_seeds {
        invoke_signed(&ix, &acc_infos, &[seeds])
    } else {
        invoke(&ix, &acc_infos)
    }
}
