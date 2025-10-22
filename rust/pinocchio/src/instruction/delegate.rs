use pinocchio::{account_info::AccountInfo, instruction::{Seed, Signer}, msg, program_error::ProgramError, pubkey, pubkey::find_program_address, sysvars::{rent::Rent, Sysvar}, ProgramResult};
use pinocchio::pubkey::Pubkey;
use pinocchio_pubkey::pubkey;
use pinocchio_system::instructions::{Assign, CreateAccount};

use crate::utils::get_signer_seeds;
use crate::{
    consts::BUFFER,
    types::DelegateConfig,
    utils::close_pda_acc,
};

// Helper: convert u64 to decimal string without heap allocation
fn dec_str_from_u64<'a>(mut n: u64, buf: &'a mut [u8; 21]) -> &'a str {
    if n == 0 {
        buf[20] = b'0';
        // SAFETY: writing only ASCII digits
        return unsafe { core::str::from_utf8_unchecked(&buf[20..21]) };
    }
    let mut i = 21usize;
    while n > 0 {
        i -= 1;
        buf[i] = b'0' + (n % 10) as u8;
        n /= 10;
    }
    // SAFETY: buffer contains only ASCII digits
    unsafe { core::str::from_utf8_unchecked(&buf[i..21]) }
}

#[allow(clippy::cloned_ref_to_slice_refs)]
pub fn delegate_account(
    accounts: &[&AccountInfo],
    pda_seeds: Signer,
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

    // Debug: print incoming PDA seeds without using formatting
    // msg!("PDA seeds:");
    // let mut num_buf = [0u8; 21];
    // let count_str = dec_str_from_u64(pda_seeds.len() as u64, &mut num_buf);
    // msg!("count:");
    // msg!(count_str);
    // for (i, seed) in pda_seeds.iter().enumerate() {
    //     msg!("seed_index:");
    //     let idx_str = dec_str_from_u64(i as u64, &mut num_buf);
    //     msg!(idx_str);
    //     msg!("seed_len:");
    //     let len_str = dec_str_from_u64(seed.len() as u64, &mut num_buf);
    //     msg!(len_str);
    //     msg!("seed_bytes:");
    //     for b in (*seed).iter() {
    //         let b_str = dec_str_from_u64(*b as u64, &mut num_buf);
    //         msg!(b_str);
    //     }
    // }

    // Buffer PDA seeds
    let buffer_seeds: &[&[u8]] = &[BUFFER, pda_acc.key().as_ref()];

    // Bumps
    // let (_, delegate_account_bump) = find_program_address(pda_seeds, owner_program.key());
    let (_, buffer_pda_bump) = find_program_address(buffer_seeds, owner_program.key());

    // Delegate signer seeds
    // let delegate_signer_seeds = get_signer_seeds(pda_seeds, delegate_account_bump)?;

    // Buffer signer seeds
    let buffer_bump_slice = [buffer_pda_bump];
    let buffer_seed_binding = [
        Seed::from(b"buffer"),
        Seed::from(pda_acc.key().as_ref()),
        Seed::from(&buffer_bump_slice),
    ];
    let buffer_signer_seeds = Signer::from(&buffer_seed_binding);

    // Single data_len and rent lookup
    let data_len = pda_acc.data_len();
    let rent_lamports = Rent::get()?.minimum_balance(data_len);

    msg!("Creating buffer PDA account");

    // Create Buffer PDA
    CreateAccount {
        from: payer,
        to: buffer_acc,
        lamports: rent_lamports,
        space: data_len as u64,
        owner: owner_program.key(),
    }
    .invoke_signed(&[buffer_signer_seeds])?;

    msg!("Buffer Created");

    // Copy delegated PDA -> buffer, then zero delegated PDA
    {
        let pda_ro = pda_acc.try_borrow_data()?;
        let mut buf_data = buffer_acc.try_borrow_mut_data()?;
        buf_data.copy_from_slice(&pda_ro);
    }
    {
        let mut pda_mut = pda_acc.try_borrow_mut_data()?;
        for b in pda_mut.iter_mut().take(data_len) {
            *b = 0;
        }
    }

    msg!("Assigning owner");

    // Assign delegated PDA to system if needed, then to delegation program

    // let (address_match, address_bump) = find_program_address(pda_seeds, owner_program.key());
    // if address_match.eq(pda_acc.key()) {
    //     msg!("address match");
    //     pubkey::log(&address_match);
    // }else {
    //     msg!("Address mismatch");
    //     pubkey::log(&address_match);
    //     pubkey::log(&pda_acc.key());
    //     if owner_program.key().eq(&pubkey!("5iC4wKZizyxrKh271Xzx3W4Vn2xUyYvSGHeoB2mdw5HA")) {
    //         msg!("Owner program key matches expected delegation program ID");
    //     }
    // }
    //
    let mint_key = pubkey!("4j7YNpHU8LsNrPKW231eUk3QWasjpZ4FMfT1oL7Zpr7z");
    let (address_match, address_bump) = find_program_address(&[payer.key().as_slice(), mint_key.as_slice()], owner_program.key());
    let bump_slice = [address_bump];
    let seeds = [
        Seed::from(payer.key().as_slice()),
        Seed::from(mint_key.as_slice()),
        // Mint: 4j7YNpHU8LsNrPKW231eUk3QWasjpZ4FMfT1oL7Zpr7z
        Seed::from(&bump_slice),
    ];
    let pda_signer_seeds = Signer::from(&seeds);

    if address_match.eq(pda_acc.key()) {
        msg!("address match");
        pubkey::log(&address_match);
    }
    pubkey::log(&payer.key());
    pubkey::log(&mint_key);

    // let delegate_signer_seeds = pda_seeds;
    let delegate_signer_seeds = pda_signer_seeds;

    let current_owner = unsafe { pda_acc.owner() };
    if current_owner != system_program.key() {
        Assign {
            account: pda_acc,
            owner: system_program.key(),
        }
        .invoke_signed(&[delegate_signer_seeds])?;
    }
    // let current_owner = unsafe { pda_acc.owner() };
    // if current_owner != &DELEGATION_PROGRAM_ID {
    //     Assign {
    //         account: pda_acc,
    //         owner: &DELEGATION_PROGRAM_ID,
    //     }
    //     .invoke_signed(&[delegate_signer_seeds.clone()])?;
    // }

    msg!("Delegating account via CPI");
    //
    // // Delegate
    // let delegate_args = DelegateAccountArgs {
    //     commit_frequency_ms: config.commit_frequency_ms,
    //     seeds: pda_seeds,
    //     validator: config.validator,
    // };

    // cpi_delegate(
    //     payer,
    //     pda_acc,
    //     owner_program,
    //     buffer_acc,
    //     delegation_record,
    //     delegation_metadata,
    //     system_program,
    //     delegate_args,
    //     delegate_signer_seeds,
    // )?;

    // Close buffer PDA back to payer to reclaim lamports

    msg!("Closing buffer PDA account");

    close_pda_acc(payer, buffer_acc, system_program)?;

    Ok(())
}
