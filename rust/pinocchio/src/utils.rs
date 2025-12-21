use crate::{
    consts::DELEGATION_PROGRAM_ID,
    types::{DelegateAccountArgs, MAX_DELEGATE_ACCOUNT_ARGS_SIZE},
};
use core::mem::MaybeUninit;
use pinocchio::pubkey::MAX_SEEDS;
use pinocchio::{
    account_info::AccountInfo,
    cpi::{invoke_signed, MAX_CPI_ACCOUNTS},
    instruction::{AccountMeta, Instruction, Seed, Signer},
    program_error::ProgramError,
};

#[inline(always)]
pub fn empty_seed<'a>() -> Seed<'a> {
    Seed::from(&[])
}

#[inline(always)]
pub fn make_seed_buf<'a>() -> [Seed<'a>; MAX_SEEDS] {
    let mut buf: [MaybeUninit<Seed<'a>>; MAX_SEEDS] =
        unsafe { MaybeUninit::uninit().assume_init() };

    let mut i = 0;
    while i < MAX_SEEDS {
        buf[i].write(empty_seed());
        i += 1;
    }

    unsafe { core::mem::transmute_copy::<_, [Seed<'a>; MAX_SEEDS]>(&buf) }
}

pub fn close_pda_acc(payer: &AccountInfo, pda_acc: &AccountInfo) -> Result<(), ProgramError> {
    unsafe {
        *payer.borrow_mut_lamports_unchecked() += *pda_acc.borrow_lamports_unchecked();
        *pda_acc.borrow_mut_lamports_unchecked() = 0;
    }

    pda_acc
        .realloc(0, false)
        .map_err(|_| ProgramError::AccountDataTooSmall)?;
    unsafe { pda_acc.assign(&pinocchio_system::ID) };

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
            .write(AccountMeta::new(pda_acc.key(), true, true));
        account_metas
            .get_unchecked_mut(2)
            .write(AccountMeta::readonly(owner_program.key()));
        account_metas
            .get_unchecked_mut(3)
            .write(AccountMeta::new(buffer_acc.key(), true, false));
        account_metas.get_unchecked_mut(4).write(AccountMeta::new(
            delegation_record.key(),
            true,
            false,
        ));
        account_metas.get_unchecked_mut(5).write(AccountMeta::new(
            delegation_metadata.key(),
            true,
            false,
        ));
        account_metas
            .get_unchecked_mut(6)
            .write(AccountMeta::readonly(&pinocchio_system::ID));
    }

    // Prepare instruction data with 8-byte discriminator prefix followed by serialized args
    let mut data = [0u8; 8 + MAX_DELEGATE_ACCOUNT_ARGS_SIZE];

    // Serialize args into the slice after the discriminator
    let args_slice = delegate_args.try_to_slice(&mut data[8..])?;
    let total_len = 8 + args_slice.len();
    let data_slice = &data[..total_len];

    let instruction = Instruction {
        program_id: &DELEGATION_PROGRAM_ID,
        accounts: unsafe {
            core::slice::from_raw_parts(account_metas.as_ptr() as *const AccountMeta, num_accounts)
        },
        data: data_slice,
    };

    let acc_infos = [
        payer,
        pda_acc,
        owner_program,
        buffer_acc,
        delegation_record,
        delegation_metadata,
    ];

    invoke_signed(&instruction, &acc_infos, &[signer_seeds])?;
    Ok(())
}

pub fn create_schedule_commit_ix<'a>(
    payer: &'a AccountInfo,
    account_infos: &'a [AccountInfo],
    magic_context: &'a AccountInfo,
    magic_program: &'a AccountInfo,
    allow_undelegation: bool,
    account_metas: &'a mut [MaybeUninit<AccountMeta<'a>>],
) -> Result<Instruction<'a, 'a, 'a, 'a>, ProgramError> {
    let num_accounts = 2 + account_infos.len();

    if num_accounts > account_metas.len() {
        return Err(ProgramError::InvalidArgument);
    }

    const COMMIT_AND_UNDELEGATE: [u8; 4] = [2, 0, 0, 0];
    const COMMIT: [u8; 4] = [1, 0, 0, 0];

    let instruction_data = if allow_undelegation {
        &COMMIT_AND_UNDELEGATE
    } else {
        &COMMIT
    };

    unsafe {
        account_metas.get_unchecked_mut(0).write(AccountMeta {
            pubkey: payer.key(),
            is_signer: true,
            // Do not escalate privileges: mirror the actual writability of the payer account
            is_writable: payer.is_writable(),
        });

        account_metas.get_unchecked_mut(1).write(AccountMeta {
            pubkey: magic_context.key(),
            is_signer: false,
            is_writable: true,
        });

        for i in 0..account_infos.len() {
            let a = account_infos.get_unchecked(i);
            account_metas.get_unchecked_mut(2 + i).write(AccountMeta {
                pubkey: a.key(),
                is_signer: a.is_signer(),
                is_writable: a.is_writable(),
            });
        }
    }

    let ix = Instruction {
        program_id: magic_program.key(),
        accounts: unsafe {
            core::slice::from_raw_parts(account_metas.as_ptr() as *const AccountMeta, num_accounts)
        },
        data: instruction_data,
    };

    Ok(ix)
}
