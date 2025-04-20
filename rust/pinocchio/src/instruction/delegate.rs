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
    let (_, delegate_account_bump) = pubkey::find_program_address(pda_seeds, &crate::ID);
    let (_, buffer_pda_bump) = pubkey::find_program_address(buffer_seeds, &crate::ID);

    let seeds_vec: Vec<&[u8]> = pda_seeds.to_vec();
    let delegate_pda_seeds = pda_seeds.iter().map(|&slice| slice.to_vec()).collect();

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
        owner: &crate::ID,
    }
    .invoke_signed(&[buffer_signer_seeds.clone()])?;

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
        lamports: Rent::get()?.minimum_balance(pda_acc.data_len()),
        space: pda_acc.data_len() as u64, //PDA acc length
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

#[cfg(test)]
mod tests {
    use pinocchio::{
        account_info::AccountInfo,
        instruction::{AccountMeta, Seed, Signer},
        program_error::ProgramError,
        pubkey,
        sysvars::{rent::Rent, Sysvar},
        ProgramResult,
    };

    use crate::{
        consts::{BUFFER, DELEGATION_PROGRAM_ID},
        instruction::delegate_account,
        types::{DelegateAccountArgs, DelegateConfig},
        utils::{close_pda_acc, cpi_delegate, get_seeds},
    };

    #[test]
    fn test_delegate_accounts() {
        //Step 1 -make accounts
        pub const PAYER: Pubkey = pubkey!("41LzznNicELmc5iCR9Jxke62a3v1VhzpBYodQF5AQwHX");
        //PDA account
        let (test_pda, _test_bump) =
            Pubkey::find_program_address(&["test".as_bytes()], &DELEGATION_PROGRAM_ID);

        //buffer account
        let (buffer_pda, _buffer_bump) = Pubkey::find_program_address(
            &[
                BUFFER.as_bytes(),
                random_paper_hash[..PAPER_SEED_HASH_LEN].as_ref(),
            ],
            &DELEGATION_PROGRAM_ID,
        );
        //Owner Program
        pub const OWNER_PROGRAM: Pubkey = pubkey!("RSC35cbUwspG38apwCszUEu6hps5t9UmGRt8P3oVLyD");
        let delegation_record = delegation_record_pda_from_delegated_account(&test_pda);
        let delegation_metadata = delegation_metadata_pda_from_delegated_account(&test_pda);
        let (system_program, system_account) = program::keyed_account_for_system_program();

        pub struct TestData {
            counter: u8,
        }

        impl TestData {
            const LEN: usize = core::mem::size_of::<TestData>();
        }

        let account_metas = vec![
            AccountMeta::new(PAYER, true, true),
            AccountMeta::new(test_pda, true, false),
            AccountMeta::new(buffer_pda, true, false),
            AccountMeta::readonly(OWNER_PROGRAM),
            AccountMeta::new(delegation_record, true, false),
            AccountMeta::new(delegation_metadata, true, false),
            AccountMeta::readonly(system_program),
        ];

        //Initialize the accounts
        let payer_account = Account::new(1 * LAMPORTS_PER_SOL, 0, &system_program);
        let mut pda_account = Account::new(1 * LAMPORTS_PER_SOL, TestData::LEN, &PROGRAM);
        let buffer_account = Account::new(0, 0, &system_program);
        let owner_account = Account::new(1 * LAMPORTS_PER_SOL, 0, &system_program);
        let record_account = Account::new(0, 0, &DELEGATION_PROGRAM_ID);
        let metadata_account = Account::new(0, 0, &DELEGATION_PROGRAM_ID);

        test_pda.data = vec![3 as u8];
        let account_infos = &vec![
            (&PAYER, payer_account.clone()),
            (&test_pda, pda_account.clone()),
            (&buffer_pda, buffer_account.clone()),
            (&OWNER_PROGRAM, owner_account.clone()),
            (&delegation_record, record_account.clone()),
            (&delegation_metadata, metadata_account.clone()),
            (&system_program, system_account.clone()),
        ];

        let accounts = [
            AccountInfo::from(account_infos[0]),
            AccountInfo::from(account_infos[1]),
            AccountInfo::from(account_infos[2]),
            AccountInfo::from(account_infos[3]),
            AccountInfo::from(account_infos[4]),
            AccountInfo::from(account_infos[5]),
            AccountInfo::from(account_infos[6]),
        ];

        //Step 2 - make seeds and config

        let seeds = &["test".as_bytes(), &[bump]];
        let config = DelegateConfig {
            commit_frequency_ms: 3000,
            validator: None,
        };

        //Call delegate_account().
        let result = delegate_account(&accounts, seeds, config);
        assert!(result.is_ok());

        //asserts
        //cpi_delegate() was invoked with expected args.
        //Account data was copied to buffer.
        //PDA was re-created.
    }
}
