use core::mem::MaybeUninit;

use crate::spl::consts::ESPL_TOKEN_PROGRAM_ID;
use crate::spl::EphemeralSplDiscriminator;
use pinocchio::cpi::{invoke, invoke_signed, Signer, MAX_CPI_ACCOUNTS};
use pinocchio::instruction::{InstructionAccount, InstructionView};
use pinocchio::{AccountView, ProgramResult};

/// Undelegate an ephemeral ATA permission.
pub fn undelegate_ephemeral_ata_permission(
    payer: AccountView,
    eata: AccountView,
    permission: AccountView,
    permission_program: AccountView,
    magic_program: AccountView,
    magic_context: AccountView,
    signer_seeds: Option<Signer<'_, '_>>,
) -> ProgramResult {
    let mut account_metas =
        [const { MaybeUninit::<InstructionAccount>::uninit() }; MAX_CPI_ACCOUNTS];
    let num_accounts = 6;

    unsafe {
        account_metas
            .get_unchecked_mut(0)
            .write(InstructionAccount::readonly_signer(payer.address()));
        account_metas
            .get_unchecked_mut(1)
            .write(InstructionAccount::readonly(eata.address()));
        account_metas
            .get_unchecked_mut(2)
            .write(InstructionAccount::writable(permission.address()));
        account_metas
            .get_unchecked_mut(3)
            .write(InstructionAccount::readonly(permission_program.address()));
        account_metas
            .get_unchecked_mut(4)
            .write(InstructionAccount::readonly(magic_program.address()));
        account_metas
            .get_unchecked_mut(5)
            .write(InstructionAccount::writable(magic_context.address()));
    }

    let acc_infos: [&AccountView; 6] = [
        &payer,
        &eata,
        &permission,
        &permission_program,
        &magic_program,
        &magic_context,
    ];

    let data: [u8; 1] = [EphemeralSplDiscriminator::UndelegateEphemeralAtaPermission as u8];

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
