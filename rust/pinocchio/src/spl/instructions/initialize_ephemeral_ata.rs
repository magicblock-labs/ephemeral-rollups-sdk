use core::mem::MaybeUninit;

use crate::spl::consts::ESPL_TOKEN_PROGRAM_ID;
use crate::spl::EphemeralSplDiscriminator;
use pinocchio::cpi::{invoke, invoke_signed, Signer, MAX_CPI_ACCOUNTS};
use pinocchio::instruction::{InstructionAccount, InstructionView};
use pinocchio::{AccountView, ProgramResult};

/// Initialize an ephemeral ATA.
pub fn initialize_ephemeral_ata(
    payer: &AccountView,
    eata: &AccountView,
    user: &AccountView,
    mint: &AccountView,
    system_program: &AccountView,
    eata_bump: u8,
    signer_seeds: Option<Signer<'_, '_>>,
) -> ProgramResult {
    let mut account_metas =
        [const { MaybeUninit::<InstructionAccount>::uninit() }; MAX_CPI_ACCOUNTS];
    let num_accounts = 5;

    unsafe {
        account_metas
            .get_unchecked_mut(0)
            .write(InstructionAccount::writable(eata.address()));
        account_metas
            .get_unchecked_mut(1)
            .write(InstructionAccount::writable_signer(payer.address()));
        account_metas
            .get_unchecked_mut(2)
            .write(InstructionAccount::readonly(user.address()));
        account_metas
            .get_unchecked_mut(3)
            .write(InstructionAccount::readonly(mint.address()));
        account_metas
            .get_unchecked_mut(4)
            .write(InstructionAccount::readonly(system_program.address()));
    }

    let acc_infos: [&AccountView; 5] = [eata, payer, user, mint, system_program];

    let data: [u8; 2] = [
        EphemeralSplDiscriminator::InitializeEphemeralAta as u8,
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
