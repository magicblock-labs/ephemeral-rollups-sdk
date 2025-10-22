use core::mem::MaybeUninit;
use pinocchio::{
    account_info::AccountInfo,
    cpi::{invoke_signed, MAX_CPI_ACCOUNTS},
    instruction::{AccountMeta, Instruction, Seed, Signer},
    msg,
    program_error::ProgramError,
};

use crate::{
    consts::DELEGATION_PROGRAM_ID,
    types::{DelegateAccountArgs, MAX_DELEGATE_ACCOUNT_ARGS_SIZE},
};

// Helper: convert u64 to decimal string without heap allocation (no_std-friendly)
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

#[inline(always)]
pub fn get_seeds<'a>(seeds_slice: &[&'a [u8]]) -> Result<&'a [Seed<'a>], ProgramError> {
    let num_seeds = seeds_slice.len();

    if num_seeds > MAX_CPI_ACCOUNTS {
        return Err(ProgramError::InvalidArgument);
    }

    if num_seeds == 0 {
        return Ok(&[]);
    }

    const UNINIT_SEED: MaybeUninit<Seed> = MaybeUninit::<Seed>::uninit();
    let mut seeds = [UNINIT_SEED; MAX_CPI_ACCOUNTS];

    for i in 0..num_seeds {
        unsafe {
            // SAFETY: i is less than len(seeds_slice) and num_seeds <= MAX_CPI_ACCOUNTS
            let seed_bytes = seeds_slice.get_unchecked(i);

            // SAFETY: i is less than MAX_CPI_ACCOUNTS
            seeds.get_unchecked_mut(i).write(Seed::from(*seed_bytes));
        }
    }

    unsafe {
        // SAFETY: num_seeds <= MAX_CPI_ACCOUNTS and we've initialized the first num_seeds elements
        Ok(core::slice::from_raw_parts(
            seeds.as_ptr() as *const Seed,
            num_seeds,
        ))
    }
}

#[inline(always)]
pub fn get_signer_seeds<'a, 'b>(
    seeds_slice: &[&'a [u8]],
    bump: u8,
) -> Result<Signer<'a, 'b>, ProgramError> {
    let num_seeds = seeds_slice.len();
    if num_seeds + 1 > MAX_CPI_ACCOUNTS {
        return Err(ProgramError::InvalidArgument);
    }

    let mut tmp: [MaybeUninit<Seed>; MAX_CPI_ACCOUNTS] =
        [const { MaybeUninit::<Seed>::uninit() }; MAX_CPI_ACCOUNTS];

    unsafe {
        for i in 0..num_seeds {
            let seed_bytes = seeds_slice.get_unchecked(i);
            tmp.get_unchecked_mut(i).write(Seed::from(*seed_bytes));
        }

        let bump_slice: &[u8] = &[bump];
        tmp.get_unchecked_mut(num_seeds)
            .write(Seed::from(bump_slice));

        let all_seeds = core::slice::from_raw_parts(tmp.as_ptr() as *const Seed, num_seeds + 1);

        // Debug: print all_seeds before creating the Signer
        // msg!("all_seeds:");
        // let mut num_buf = [0u8; 21];
        // msg!("count:");
        // let count_str = dec_str_from_u64((num_seeds as u64) + 1, &mut num_buf);
        // msg!(count_str);
        // for i in 0..(num_seeds + 1) {
        //     msg!("seed_index:");
        //     let idx_str = dec_str_from_u64(i as u64, &mut num_buf);
        //     msg!(idx_str);
        //     let seed_ref = all_seeds.get_unchecked(i);
        //     let seed_bytes: &[u8] = &*seed_ref;
        //     msg!("seed_len:");
        //     let len_str = dec_str_from_u64(seed_bytes.len() as u64, &mut num_buf);
        //     msg!(len_str);
        //     msg!("seed_bytes:");
        //     for b in seed_bytes.iter() {
        //         let b_str = dec_str_from_u64(*b as u64, &mut num_buf);
        //         msg!(b_str);
        //     }
        // }

        Ok(Signer::from(all_seeds))
    }
}

pub fn close_pda_acc(
    payer: &AccountInfo,
    pda_acc: &AccountInfo,
    system_program: &AccountInfo,
) -> Result<(), ProgramError> {
    unsafe {
        *payer.borrow_mut_lamports_unchecked() += *pda_acc.borrow_lamports_unchecked();
        *pda_acc.borrow_mut_lamports_unchecked() = 0;
    }

    pda_acc
        .realloc(0, false)
        .map_err(|_| ProgramError::AccountDataTooSmall)?;
    unsafe { pda_acc.assign(system_program.key()) };

    Ok(())
}

#[allow(clippy::too_many_arguments)]
pub fn cpi_delegate(
    payer: &AccountInfo,
    pda_acc: &AccountInfo,
    owner_program: &AccountInfo,
    buffer_acc: &AccountInfo,
    delegation_record: &AccountInfo,
    delegation_metadata: &AccountInfo,
    system_program: &AccountInfo,
    delegate_args: DelegateAccountArgs,
    signer_seeds: Signer<'_, '_>,
) -> Result<(), ProgramError> {
    const UNINIT_META: MaybeUninit<AccountMeta> = MaybeUninit::<AccountMeta>::uninit();
    let mut account_metas = [UNINIT_META; MAX_CPI_ACCOUNTS];

    let num_accounts = 7;

    unsafe {
        // SAFETY: num_accounts <= MAX_CPI_ACCOUNTS
        account_metas
            .get_unchecked_mut(0)
            .write(AccountMeta::new(payer.key(), true, true));
        account_metas
            .get_unchecked_mut(1)
            .write(AccountMeta::new(pda_acc.key(), true, false));
        account_metas
            .get_unchecked_mut(2)
            .write(AccountMeta::readonly(owner_program.key()));
        account_metas
            .get_unchecked_mut(3)
            .write(AccountMeta::new(buffer_acc.key(), false, false));
        account_metas.get_unchecked_mut(4).write(AccountMeta::new(
            delegation_record.key(),
            true,
            false,
        ));
        account_metas
            .get_unchecked_mut(5)
            .write(AccountMeta::readonly(delegation_metadata.key()));
        account_metas
            .get_unchecked_mut(6)
            .write(AccountMeta::readonly(system_program.key()));
    }

    let mut data = [0u8; MAX_DELEGATE_ACCOUNT_ARGS_SIZE];

    let serialized_data = delegate_args.try_to_slice(&mut data)?;

    let instruction = Instruction {
        program_id: &DELEGATION_PROGRAM_ID,
        accounts: unsafe {
            core::slice::from_raw_parts(account_metas.as_ptr() as *const AccountMeta, num_accounts)
        },
        data: serialized_data,
    };

    let acc_infos = [
        payer,
        pda_acc,
        owner_program,
        buffer_acc,
        delegation_record,
        delegation_metadata,
        system_program,
    ];

    invoke_signed(&instruction, &acc_infos, &[signer_seeds])?;
    Ok(())
}

pub fn create_schedule_commit_ix<'a>(
    payer: &'a AccountInfo,
    account_infos: &'a [AccountInfo],
    magic_context: &'a AccountInfo,
    allow_undelegation: bool,
) -> Result<(&'a [u8], &'a [AccountMeta<'a>]), ProgramError> {
    let num_accounts = 2 + account_infos.len();

    if num_accounts > MAX_CPI_ACCOUNTS {
        return Err(ProgramError::InvalidArgument);
    }

    const ALLOW_UNDELEGATION_DATA: [u8; 4] = [2, 0, 0, 0];
    const DISALLOW_UNDELEGATION_DATA: [u8; 4] = [1, 0, 0, 0];

    let instruction_data = if allow_undelegation {
        &ALLOW_UNDELEGATION_DATA
    } else {
        &DISALLOW_UNDELEGATION_DATA
    };

    const UNINIT_META: MaybeUninit<AccountMeta> = MaybeUninit::<AccountMeta>::uninit();
    let mut account_metas = [UNINIT_META; MAX_CPI_ACCOUNTS];

    unsafe {
        // SAFETY: num_accounts <= MAX_CPI_ACCOUNTS
        account_metas
            .get_unchecked_mut(0)
            .write(AccountMeta::new(payer.key(), true, true));
        account_metas.get_unchecked_mut(1).write(AccountMeta::new(
            magic_context.key(),
            true,
            false,
        ));

        for i in 0..account_infos.len() {
            let account = account_infos.get_unchecked(i);
            account_metas
                .get_unchecked_mut(2 + i)
                .write(AccountMeta::new(account.key(), true, true));
        }
    }

    Ok((instruction_data, unsafe {
        core::slice::from_raw_parts(account_metas.as_ptr() as *const AccountMeta, num_accounts)
    }))
}
