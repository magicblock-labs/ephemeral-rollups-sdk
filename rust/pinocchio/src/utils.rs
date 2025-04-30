use pinocchio::{
    account_info::AccountInfo,
    cpi::invoke_signed,
    instruction::{AccountMeta, Instruction, Seed, Signer},
    program_error::ProgramError,
};

use crate::{consts::DELEGATION_PROGRAM_ID, types::DelegateAccountArgs};

#[inline(always)]
pub fn get_seeds<'a>(seeds_vec: Vec<&'a [u8]>) -> Result<Vec<Seed<'a>>, ProgramError> {
    let mut seeds: Vec<Seed<'a>> = Vec::with_capacity(seeds_vec.len() + 1);

    for seed in seeds_vec {
        seeds.push(Seed::from(seed));
    }

    Ok(seeds)
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
    let account_metas = vec![
        AccountMeta::new(payer.key(), true, true),
        AccountMeta::new(pda_acc.key(), true, false),
        AccountMeta::readonly(owner_program.key()),
        AccountMeta::new(buffer_acc.key(), false, false),
        AccountMeta::new(delegation_record.key(), true, false),
        AccountMeta::readonly(delegation_metadata.key()),
        AccountMeta::readonly(system_program.key()),
    ];

    let mut data: Vec<u8> = vec![0u8; 8];
    let serialized_seeds = delegate_args
        .try_to_vec()
        .map_err(|_op| ProgramError::BorshIoError)?;
    data.extend_from_slice(&serialized_seeds);

    let instruction = Instruction {
        program_id: &DELEGATION_PROGRAM_ID,
        accounts: &account_metas,
        data: &data,
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
) -> (Vec<u8>, Vec<AccountMeta<'a>>) {
    let instruction_data: Vec<u8> = if allow_undelegation {
        vec![2, 0, 0, 0]
    } else {
        vec![1, 0, 0, 0]
    };
    let mut account_metas = vec![
        AccountMeta::new(payer.key(), true, true),
        AccountMeta::new(magic_context.key(), true, false),
    ];
    account_metas.extend(
        account_infos
            .iter()
            .map(|acc| AccountMeta::new(acc.key(), true, true)),
    );
    (instruction_data, account_metas)
}
