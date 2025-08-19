use core::mem::MaybeUninit;
use pinocchio::{
    account_info::AccountInfo,
    cpi::MAX_CPI_ACCOUNTS,
    instruction::{Seed, Signer},
    program_error::ProgramError,
    pubkey::find_program_address,
    sysvars::{rent::Rent, Sysvar},
    ProgramResult,
};
use pinocchio_system::instructions::CreateAccount;

use crate::{consts::DELEGATION_PROGRAM_ID, utils::get_seeds};

#[allow(clippy::cloned_ref_to_slice_refs)]
pub fn undelegate(accounts: &[AccountInfo], account_signer_seeds: &[&[u8]]) -> ProgramResult {
    let [payer, delegated_acc, owner_program, buffer_acc, _system_program] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };

    if !buffer_acc.is_signer() {
        return Err(ProgramError::MissingRequiredSignature);
    }

    //Find delegate
    let (_, delegate_account_bump) =
        find_program_address(account_signer_seeds, &DELEGATION_PROGRAM_ID);

    //Get Delegated Pda Signer Seeds
    let binding = &[delegate_account_bump];
    let delegate_bump = Seed::from(binding);
    let delegate_seeds = get_seeds(account_signer_seeds)?;

    let num_seeds = delegate_seeds.len() + 1;
    if num_seeds > MAX_CPI_ACCOUNTS {
        return Err(ProgramError::InvalidArgument);
    }

    const UNINIT_SEED: MaybeUninit<Seed> = MaybeUninit::<Seed>::uninit();
    let mut combined_seeds = [UNINIT_SEED; MAX_CPI_ACCOUNTS];

    unsafe {
        for i in 0..num_seeds - 1 {
            let seed = delegate_seeds.get_unchecked(i);
            combined_seeds
                .get_unchecked_mut(i)
                .write(Seed::from(seed.as_ref()));
        }

        combined_seeds
            .get_unchecked_mut(num_seeds - 1)
            .write(delegate_bump);
    }

    let all_delegate_seeds =
        unsafe { core::slice::from_raw_parts(combined_seeds.as_ptr() as *const Seed, num_seeds) };

    let delegate_signer_seeds = Signer::from(all_delegate_seeds);

    //Create the original PDA Account Delegated
    CreateAccount {
        from: payer,
        to: delegated_acc,
        lamports: Rent::get()?.minimum_balance(buffer_acc.data_len()),
        space: buffer_acc.data_len() as u64,
        owner: owner_program.key(),
    }
    .invoke_signed(&[delegate_signer_seeds.clone()])?;

    let mut data = delegated_acc.try_borrow_mut_data()?;
    let buffer_data = buffer_acc.try_borrow_data()?;
    (*data).copy_from_slice(&buffer_data);

    Ok(())
}
