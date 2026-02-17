use core::mem::MaybeUninit;

use crate::spl::consts::ESPL_TOKEN_PROGRAM_ID;
use crate::spl::EphemeralSplDiscriminator;
use pinocchio::cpi::{invoke, invoke_signed, Signer, MAX_CPI_ACCOUNTS};
use pinocchio::instruction::{InstructionAccount, InstructionView};
use pinocchio::{AccountView, ProgramResult};

/// Undelegate an ephemeral ATA.
pub fn undelegate_ephemeral_ata(
    payer: AccountView,
    user_ata: AccountView,
    eata: AccountView,
    magic_context: AccountView,
    magic_program: AccountView,
    signer_seeds: Option<Signer<'_, '_>>,
) -> ProgramResult {
    let mut account_metas =
        [const { MaybeUninit::<InstructionAccount>::uninit() }; MAX_CPI_ACCOUNTS];
    let num_accounts = 5;

    unsafe {
        account_metas
            .get_unchecked_mut(0)
            .write(InstructionAccount::readonly_signer(payer.address()));
        account_metas
            .get_unchecked_mut(1)
            .write(InstructionAccount::writable(user_ata.address()));
        account_metas
            .get_unchecked_mut(2)
            .write(InstructionAccount::readonly(eata.address()));
        account_metas
            .get_unchecked_mut(3)
            .write(InstructionAccount::writable(magic_context.address()));
        account_metas
            .get_unchecked_mut(4)
            .write(InstructionAccount::readonly(magic_program.address()));
    }

    let acc_infos: [&AccountView; 5] = [&payer, &user_ata, &eata, &magic_context, &magic_program];

    let data: [u8; 1] = [EphemeralSplDiscriminator::UndelegateEphemeralAta as u8];

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
