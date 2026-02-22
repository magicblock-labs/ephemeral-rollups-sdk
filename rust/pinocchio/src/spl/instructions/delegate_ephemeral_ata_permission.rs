use core::mem::MaybeUninit;

use crate::spl::consts::ESPL_TOKEN_PROGRAM_ID;
use crate::spl::EphemeralSplDiscriminator;
use pinocchio::cpi::{invoke, invoke_signed, Signer, MAX_CPI_ACCOUNTS};
use pinocchio::instruction::{InstructionAccount, InstructionView};
use pinocchio::{AccountView, ProgramResult};

/// Delegate an ephemeral ATA permission.
#[allow(clippy::too_many_arguments)]
pub fn delegate_ephemeral_ata_permission(
    payer: &AccountView,
    eata: &AccountView,
    permission: &AccountView,
    permission_program: &AccountView,
    system_program: &AccountView,
    delegation_buffer: &AccountView,
    delegation_record: &AccountView,
    delegation_metadata: &AccountView,
    delegation_program: &AccountView,
    validator: &AccountView,
    eata_bump: u8,
    signer_seeds: Option<Signer<'_, '_>>,
) -> ProgramResult {
    let mut account_metas =
        [const { MaybeUninit::<InstructionAccount>::uninit() }; MAX_CPI_ACCOUNTS];
    let num_accounts = 10;

    unsafe {
        account_metas
            .get_unchecked_mut(0)
            .write(InstructionAccount::readonly_signer(payer.address()));
        account_metas
            .get_unchecked_mut(1)
            .write(InstructionAccount::writable(eata.address()));
        account_metas
            .get_unchecked_mut(2)
            .write(InstructionAccount::readonly(permission_program.address()));
        account_metas
            .get_unchecked_mut(3)
            .write(InstructionAccount::writable(permission.address()));
        account_metas
            .get_unchecked_mut(4)
            .write(InstructionAccount::readonly(system_program.address()));
        account_metas
            .get_unchecked_mut(5)
            .write(InstructionAccount::writable(delegation_buffer.address()));
        account_metas
            .get_unchecked_mut(6)
            .write(InstructionAccount::writable(delegation_record.address()));
        account_metas
            .get_unchecked_mut(7)
            .write(InstructionAccount::writable(delegation_metadata.address()));
        account_metas
            .get_unchecked_mut(8)
            .write(InstructionAccount::readonly(delegation_program.address()));
        account_metas
            .get_unchecked_mut(9)
            .write(InstructionAccount::readonly(validator.address()));
    }

    let acc_infos: [&AccountView; 10] = [
        payer,
        eata,
        payer,
        permission,
        system_program,
        delegation_buffer,
        delegation_record,
        delegation_metadata,
        delegation_program,
        validator,
    ];

    let data = [
        EphemeralSplDiscriminator::DelegateEphemeralAtaPermission as u8,
        eata_bump,
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
