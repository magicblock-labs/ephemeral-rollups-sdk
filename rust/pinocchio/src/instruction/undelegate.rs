use pinocchio::{
    account_info::AccountInfo,
    instruction::{Seed, Signer},
    program_error::ProgramError,
    pubkey,
    sysvars::{rent::Rent, Sysvar},
    ProgramResult,
};

use crate::utils::get_seeds;

pub fn undelegate(accounts: &[AccountInfo], account_signer_seeds: Vec<Vec<u8>>) -> ProgramResult {
    let [payer, delegated_acc, owner_program, buffer_acc, _system_program, _rest @ ..] = accounts
    else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };

    if !buffer_acc.is_signer() {
        return Err(ProgramError::MissingRequiredSignature);
    }

    //Get buffer seeds
    let account_seeds: Vec<&[u8]> = account_signer_seeds.iter().map(|v| v.as_slice()).collect();

    //Find delegate
    let (_, delegate_account_bump) = pubkey::find_program_address(&account_seeds, &crate::ID);

    //Get Delegated Pda Signer Seeds
    let binding = &[delegate_account_bump];
    let delegate_bump = Seed::from(binding);
    let mut delegate_seeds = get_seeds(account_seeds)?;
    delegate_seeds.extend_from_slice(&[delegate_bump]);
    let delegate_signer_seeds = Signer::from(delegate_seeds.as_slice());

    //Create the original PDA Account Delegated
    pinocchio_system::instructions::CreateAccount {
        from: payer,
        to: delegated_acc,
        lamports: Rent::get()?.minimum_balance(buffer_acc.data_len()),
        space: buffer_acc.data_len() as u64, //PDA acc length
        owner: &owner_program.key(),
    }
    .invoke_signed(&[delegate_signer_seeds.clone()])?;

    let mut data = delegated_acc.try_borrow_mut_data()?;
    let buffer_data = buffer_acc.try_borrow_data()?;
    (*data).copy_from_slice(&buffer_data);

    Ok(())
}
