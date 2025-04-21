use pinocchio::{
    account_info::AccountInfo,
    instruction::{Seed, Signer},
    program_error::ProgramError,
    pubkey,
    sysvars::{rent::Rent, Sysvar},
    ProgramResult,
};

use crate::{
    consts::{BUFFER, DELEGATION_PROGRAM_ID},
    types::{DelegateAccountArgs, DelegateConfig},
    utils::{close_pda_acc, cpi_delegate, get_seeds},
};

pub fn delegate_account(
    accounts: &[AccountInfo],
    pda_seeds: &[&[u8]],
    config: DelegateConfig,
) -> ProgramResult {
    let [payer, pda_acc, owner_program, buffer_acc, delegation_record, delegation_metadata, system_program] =
        accounts
    else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };

    if !payer.is_signer() {
        return Err(ProgramError::MissingRequiredSignature);
    }

    //Get buffer seeds
    let buffer_seeds: &[&[u8]] = &[BUFFER, pda_acc.key().as_ref()];

    //Find PDAs
    let (_, delegate_account_bump) = pubkey::find_program_address(pda_seeds, owner_program.key());
    let (_, buffer_pda_bump) = pubkey::find_program_address(buffer_seeds, owner_program.key());

    let seeds_vec: Vec<&[u8]> = pda_seeds.iter().map(|x| *x).collect();
    let delegate_pda_seeds: Vec<Vec<u8>> = pda_seeds.iter().map(|&s| s.to_vec()).collect();

    //Get Delegated Pda Signer Seeds
    let binding = &[delegate_account_bump];
    let delegate_bump = Seed::from(binding);
    let mut delegate_seeds = get_seeds(seeds_vec)?;
    delegate_seeds.extend_from_slice(&[delegate_bump]);
    let delegate_signer_seeds = Signer::from(delegate_seeds.as_slice());

    //Get Buffer signer seeds
    let bump = [buffer_pda_bump];
    let seed_b = [
        Seed::from(b"buffer"),
        Seed::from(pda_acc.key().as_ref()),
        Seed::from(&bump),
    ];

    let buffer_signer_seeds = Signer::from(&seed_b);

    //Create Buffer PDA account
    pinocchio_system::instructions::CreateAccount {
        from: payer,
        to: buffer_acc,
        lamports: Rent::get()?.minimum_balance(pda_acc.data_len()),
        space: pda_acc.data_len() as u64, //PDA acc length
        owner: &owner_program.key(),
    }
    .invoke_signed(&[buffer_signer_seeds])?;

    // Copy the data to the buffer PDA
    let mut buffer_data = buffer_acc.try_borrow_mut_data()?;
    let new_data = pda_acc.try_borrow_data()?.to_vec().clone();
    (*buffer_data).copy_from_slice(&new_data);
    drop(buffer_data);

    //Close Delegate PDA in preparation for CPI Delegate
    close_pda_acc(payer, pda_acc, system_program)?;

    //Create account with Delegation Account
    pinocchio_system::instructions::CreateAccount {
        from: payer,
        to: pda_acc,
        lamports: Rent::get()?.minimum_balance(buffer_acc.data_len()),
        space: buffer_acc.data_len() as u64, //PDA acc length
        owner: &DELEGATION_PROGRAM_ID,
    }
    .invoke_signed(&[delegate_signer_seeds.clone()])?;

    //Preprare delegate args
    let delegate_args = DelegateAccountArgs {
        commit_frequency_ms: config.commit_frequency_ms,
        seeds: delegate_pda_seeds,
        validator: config.validator,
    };

    cpi_delegate(
        payer,
        pda_acc,
        owner_program,
        buffer_acc,
        delegation_record,
        delegation_metadata,
        system_program,
        delegate_args,
        delegate_signer_seeds,
    )?;

    close_pda_acc(payer, buffer_acc, system_program)?;

    Ok(())
}
