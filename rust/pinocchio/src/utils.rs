use pinocchio::{
    account_info::AccountInfo,
    cpi::{invoke_signed, MAX_CPI_ACCOUNTS},
    instruction::{AccountMeta, Instruction, Seed, Signer},
    program_error::ProgramError,
};
use core::mem::MaybeUninit;

use crate::{consts::DELEGATION_PROGRAM_ID, types::DelegateAccountArgs};

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
            seeds
                .get_unchecked_mut(i)
                .write(Seed::from(*seed_bytes));
        }
    }
    
    unsafe {
        // SAFETY: num_seeds <= MAX_CPI_ACCOUNTS and we've initialized the first num_seeds elements
        Ok(core::slice::from_raw_parts(
            seeds.as_ptr() as *const Seed,
            num_seeds
        ))
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
        account_metas.get_unchecked_mut(0).write(AccountMeta::new(payer.key(), true, true));
        account_metas.get_unchecked_mut(1).write(AccountMeta::new(pda_acc.key(), true, false));
        account_metas.get_unchecked_mut(2).write(AccountMeta::readonly(owner_program.key()));
        account_metas.get_unchecked_mut(3).write(AccountMeta::new(buffer_acc.key(), false, false));
        account_metas.get_unchecked_mut(4).write(AccountMeta::new(delegation_record.key(), true, false));
        account_metas.get_unchecked_mut(5).write(AccountMeta::readonly(delegation_metadata.key()));
        account_metas.get_unchecked_mut(6).write(AccountMeta::readonly(system_program.key()));
    }
    
    let data = delegate_args.try_to_slice()?;

    let instruction = Instruction {
        program_id: &DELEGATION_PROGRAM_ID,
        accounts: unsafe {
            core::slice::from_raw_parts(account_metas.as_ptr() as *const AccountMeta, num_accounts)
        },
        data,
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
        account_metas.get_unchecked_mut(0).write(AccountMeta::new(payer.key(), true, true));
        account_metas.get_unchecked_mut(1).write(AccountMeta::new(magic_context.key(), true, false));
        
        for i in 0..account_infos.len() {
            let account = account_infos.get_unchecked(i);
            account_metas
                .get_unchecked_mut(2 + i)
                .write(AccountMeta::new(account.key(), true, true));
        }
    }
    
    Ok((
        instruction_data,
        unsafe {
            core::slice::from_raw_parts(account_metas.as_ptr() as *const AccountMeta, num_accounts)
        }
    ))
}
