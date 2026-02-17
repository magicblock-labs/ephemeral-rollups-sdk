use core::mem::MaybeUninit;

use crate::spl::consts::ESPL_TOKEN_PROGRAM_ID;
use crate::spl::EphemeralSplDiscriminator;
use pinocchio::cpi::{invoke, invoke_signed, Signer, MAX_CPI_ACCOUNTS};
use pinocchio::instruction::{InstructionAccount, InstructionView};
use pinocchio::{AccountView, ProgramResult};

/// Deposit SPL tokens into an ephemeral ATA.
#[allow(clippy::too_many_arguments)]
pub fn deposit_spl_tokens(
    authority: AccountView,
    eata: AccountView,
    vault: AccountView,
    mint: AccountView,
    user_source_token_acc: AccountView,
    vault_token_acc: AccountView,
    token_program: AccountView,
    amount: u64,
    signer_seeds: Option<Signer<'_, '_>>,
) -> ProgramResult {
    let mut account_metas =
        [const { MaybeUninit::<InstructionAccount>::uninit() }; MAX_CPI_ACCOUNTS];
    let num_accounts = 7;

    unsafe {
        account_metas
            .get_unchecked_mut(0)
            .write(InstructionAccount::writable(eata.address()));
        account_metas
            .get_unchecked_mut(1)
            .write(InstructionAccount::readonly(vault.address()));
        account_metas
            .get_unchecked_mut(2)
            .write(InstructionAccount::readonly(mint.address()));
        account_metas
            .get_unchecked_mut(3)
            .write(InstructionAccount::writable(
                user_source_token_acc.address(),
            ));
        account_metas
            .get_unchecked_mut(4)
            .write(InstructionAccount::writable(vault_token_acc.address()));
        account_metas
            .get_unchecked_mut(5)
            .write(InstructionAccount::readonly_signer(authority.address()));
        account_metas
            .get_unchecked_mut(6)
            .write(InstructionAccount::readonly(token_program.address()));
    }

    let acc_infos: [&AccountView; 7] = [
        &eata,
        &vault,
        &mint,
        &user_source_token_acc,
        &vault_token_acc,
        &authority,
        &token_program,
    ];

    let mut data: [u8; 9] = [0; 9];
    data[0] = EphemeralSplDiscriminator::DepositSplTokens as u8;
    data[1..9].copy_from_slice(&amount.to_le_bytes());

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
