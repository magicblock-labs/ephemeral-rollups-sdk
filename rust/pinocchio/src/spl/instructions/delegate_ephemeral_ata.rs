use core::mem::MaybeUninit;

use crate::spl::consts::ESPL_TOKEN_PROGRAM_ID;
use crate::spl::EphemeralSplDiscriminator;
use pinocchio::cpi::{invoke, invoke_signed, Signer, MAX_CPI_ACCOUNTS};
use pinocchio::instruction::{InstructionAccount, InstructionView};
use pinocchio::{AccountView, Address, ProgramResult};

/// Delegate an ephemeral ATA.
#[allow(clippy::too_many_arguments)]
pub fn delegate_ephemeral_ata(
    payer: AccountView,
    eata: AccountView,
    espl_token_program: AccountView,
    delegation_buffer: AccountView,
    delegation_record: AccountView,
    delegation_metadata: AccountView,
    delegation_program: AccountView,
    system_program: AccountView,
    eata_bump: u8,
    validator: Option<Address>,
    signer_seeds: Option<Signer<'_, '_>>,
) -> ProgramResult {
    let mut account_metas =
        [const { MaybeUninit::<InstructionAccount>::uninit() }; MAX_CPI_ACCOUNTS];
    let num_accounts = 8;

    unsafe {
        account_metas
            .get_unchecked_mut(0)
            .write(InstructionAccount::readonly_signer(payer.address()));
        account_metas
            .get_unchecked_mut(1)
            .write(InstructionAccount::writable(eata.address()));
        account_metas
            .get_unchecked_mut(2)
            .write(InstructionAccount::readonly(espl_token_program.address()));
        account_metas
            .get_unchecked_mut(3)
            .write(InstructionAccount::writable(delegation_buffer.address()));
        account_metas
            .get_unchecked_mut(4)
            .write(InstructionAccount::writable(delegation_record.address()));
        account_metas
            .get_unchecked_mut(5)
            .write(InstructionAccount::writable(delegation_metadata.address()));
        account_metas
            .get_unchecked_mut(6)
            .write(InstructionAccount::readonly(delegation_program.address()));
        account_metas
            .get_unchecked_mut(7)
            .write(InstructionAccount::readonly(system_program.address()));
    }

    let acc_infos: [&AccountView; 8] = [
        &payer,
        &eata,
        &espl_token_program,
        &delegation_buffer,
        &delegation_record,
        &delegation_metadata,
        &delegation_program,
        &system_program,
    ];

    let mut data = [0_u8; 34];
    data[0] = EphemeralSplDiscriminator::DelegateEphemeralAta as u8;
    data[1] = eata_bump;
    let data = if let Some(validator) = validator {
        data[2..34].copy_from_slice(validator.as_ref());
        &data
    } else {
        &data[..2]
    };

    let ix = InstructionView {
        program_id: &ESPL_TOKEN_PROGRAM_ID,
        accounts: unsafe {
            core::slice::from_raw_parts(
                account_metas.as_ptr() as *const InstructionAccount,
                num_accounts,
            )
        },
        data,
    };

    if let Some(seeds) = signer_seeds {
        invoke_signed(&ix, &acc_infos, &[seeds])
    } else {
        invoke(&ix, &acc_infos)
    }
}
