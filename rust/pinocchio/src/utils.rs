use crate::{
    consts::DELEGATION_PROGRAM_ID,
    types::{DelegateAccountArgs, MAX_DELEGATE_ACCOUNT_ARGS_SIZE},
};
use core::mem::MaybeUninit;
use pinocchio::{
    address::MAX_SEEDS,
    cpi::{invoke_signed, Seed, Signer, MAX_CPI_ACCOUNTS},
    error::ProgramError,
    instruction::{InstructionAccount, InstructionView},
    AccountView,
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

pub fn close_pda_acc(payer: &AccountView, pda_acc: &AccountView) -> Result<(), ProgramError> {
    payer.set_lamports(payer.lamports() + pda_acc.lamports());
    pda_acc.set_lamports(0);

    pda_acc
        .resize(0)
        .map_err(|_| ProgramError::AccountDataTooSmall)?;
    unsafe { pda_acc.assign(&pinocchio_system::ID) };

    Ok(())
}

#[allow(clippy::too_many_arguments)]
pub fn cpi_delegate(
    payer: &AccountView,
    pda_acc: &AccountView,
    owner_program: &AccountView,
    buffer_acc: &AccountView,
    delegation_record: &AccountView,
    delegation_metadata: &AccountView,
    system_program: &AccountView,
    delegate_args: DelegateAccountArgs,
    signer_seeds: Signer<'_, '_>,
) -> Result<(), ProgramError> {
    cpi_delegate_with_discriminator(
        0,
        payer,
        pda_acc,
        owner_program,
        buffer_acc,
        delegation_record,
        delegation_metadata,
        system_program,
        delegate_args,
        signer_seeds,
    )
}

#[allow(clippy::too_many_arguments)]
pub fn cpi_delegate_with_any_validator(
    payer: &AccountView,
    pda_acc: &AccountView,
    owner_program: &AccountView,
    buffer_acc: &AccountView,
    delegation_record: &AccountView,
    delegation_metadata: &AccountView,
    system_program: &AccountView,
    delegate_args: DelegateAccountArgs,
    signer_seeds: Signer<'_, '_>,
) -> Result<(), ProgramError> {
    cpi_delegate_with_discriminator(
        19,
        payer,
        pda_acc,
        owner_program,
        buffer_acc,
        delegation_record,
        delegation_metadata,
        system_program,
        delegate_args,
        signer_seeds,
    )
}

#[allow(clippy::too_many_arguments)]
fn cpi_delegate_with_discriminator(
    discriminator: u64,
    payer: &AccountView,
    pda_acc: &AccountView,
    owner_program: &AccountView,
    buffer_acc: &AccountView,
    delegation_record: &AccountView,
    delegation_metadata: &AccountView,
    system_program: &AccountView,
    delegate_args: DelegateAccountArgs,
    signer_seeds: Signer<'_, '_>,
) -> Result<(), ProgramError> {
    const UNINIT_ACCOUNT: MaybeUninit<InstructionAccount> =
        MaybeUninit::<InstructionAccount>::uninit();
    let mut account_metas = [UNINIT_ACCOUNT; MAX_CPI_ACCOUNTS];

    let num_accounts = 7;

    unsafe {
        // SAFETY: num_accounts <= MAX_CPI_ACCOUNTS
        account_metas
            .get_unchecked_mut(0)
            .write(InstructionAccount::writable_signer(payer.address()));
        account_metas
            .get_unchecked_mut(1)
            .write(InstructionAccount::writable_signer(pda_acc.address()));
        account_metas
            .get_unchecked_mut(2)
            .write(InstructionAccount::readonly(owner_program.address()));
        account_metas
            .get_unchecked_mut(3)
            .write(InstructionAccount::writable(buffer_acc.address()));
        account_metas
            .get_unchecked_mut(4)
            .write(InstructionAccount::writable(delegation_record.address()));
        account_metas
            .get_unchecked_mut(5)
            .write(InstructionAccount::writable(delegation_metadata.address()));
        account_metas
            .get_unchecked_mut(6)
            .write(InstructionAccount::readonly(&pinocchio_system::ID));
    }

    let mut data = [0u8; 8 + MAX_DELEGATE_ACCOUNT_ARGS_SIZE];
    data[..8].copy_from_slice(&discriminator.to_le_bytes());

    let args_slice = delegate_args.try_to_slice(&mut data[8..])?;
    let total_len = 8 + args_slice.len();
    let data_slice = &data[..total_len];

    let instruction = InstructionView {
        program_id: &DELEGATION_PROGRAM_ID,
        accounts: unsafe {
            core::slice::from_raw_parts(
                account_metas.as_ptr() as *const InstructionAccount,
                num_accounts,
            )
        },
        data: data_slice,
    };

    let acc_infos: [&AccountView; 7] = [
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

#[cfg(feature = "delegation-actions")]
pub fn serialize_delegate_with_actions_data(
    delegate_args: DelegateAccountArgs,
    actions: dlp_api::dlp::args::PostDelegationActions,
) -> Result<alloc::vec::Vec<u8>, ProgramError> {
    use alloc::vec::Vec;
    use dlp_api::dlp::args::{DelegateArgs, DelegateWithActionsArgs};
    use dlp_api::dlp::discriminator::DlpDiscriminator;
    use solana_program::pubkey::Pubkey;

    let seeds_vec: Vec<Vec<u8>> = delegate_args.seeds.iter().map(|s| s.to_vec()).collect();
    let delegate = DelegateArgs {
        commit_frequency_ms: delegate_args.commit_frequency_ms,
        seeds: seeds_vec,
        validator: delegate_args
            .validator
            .map(|v| Pubkey::new_from_array(*v.as_array())),
    };
    let args = DelegateWithActionsArgs { delegate, actions };

    let mut data = DlpDiscriminator::DelegateWithActions.to_vec();
    let payload = borsh::to_vec(&args).map_err(|_| ProgramError::InvalidInstructionData)?;
    data.extend_from_slice(&payload);
    Ok(data)
}

#[cfg(feature = "delegation-actions")]
#[allow(clippy::too_many_arguments)]
pub fn cpi_delegate_with_actions(
    payer: &AccountView,
    pda_acc: &AccountView,
    owner_program: &AccountView,
    buffer_acc: &AccountView,
    delegation_record: &AccountView,
    delegation_metadata: &AccountView,
    system_program: &AccountView,
    delegate_args: DelegateAccountArgs,
    actions: dlp_api::dlp::args::PostDelegationActions,
    action_signer_accounts: &[&AccountView],
    signer_seeds: Signer<'_, '_>,
) -> Result<(), ProgramError> {
    use pinocchio::cpi::invoke_signed_with_bounds;

    use crate::consts::MAX_POST_DELEGATION_SIGNERS;

    const MAX_ACCOUNTS: usize = 7 + MAX_POST_DELEGATION_SIGNERS;

    if action_signer_accounts.len() != actions.signers.len() {
        return Err(ProgramError::NotEnoughAccountKeys);
    }

    const UNINIT_ACCOUNT: MaybeUninit<InstructionAccount> =
        MaybeUninit::<InstructionAccount>::uninit();
    // Keep this bounded to the same static CPI cap used by `invoke_signed_with_bounds`
    // to avoid blowing the sBPF 4KB per-frame stack budget.
    let mut account_metas = [UNINIT_ACCOUNT; MAX_ACCOUNTS];

    let num_accounts = 7 + action_signer_accounts.len();
    if num_accounts > MAX_ACCOUNTS {
        return Err(ProgramError::InvalidArgument);
    }

    unsafe {
        account_metas
            .get_unchecked_mut(0)
            .write(InstructionAccount::writable_signer(payer.address()));
        account_metas
            .get_unchecked_mut(1)
            .write(InstructionAccount::writable_signer(pda_acc.address()));
        account_metas
            .get_unchecked_mut(2)
            .write(InstructionAccount::readonly(owner_program.address()));
        account_metas
            .get_unchecked_mut(3)
            .write(InstructionAccount::writable(buffer_acc.address()));
        account_metas
            .get_unchecked_mut(4)
            .write(InstructionAccount::writable(delegation_record.address()));
        account_metas
            .get_unchecked_mut(5)
            .write(InstructionAccount::writable(delegation_metadata.address()));
        account_metas
            .get_unchecked_mut(6)
            .write(InstructionAccount::readonly(&pinocchio_system::ID));
    }

    let mut i = 0;
    while i < action_signer_accounts.len() {
        unsafe {
            account_metas
                .get_unchecked_mut(7 + i)
                .write(InstructionAccount::readonly_signer(
                    action_signer_accounts[i].address(),
                ));
        }
        i += 1;
    }

    let data = serialize_delegate_with_actions_data(delegate_args, actions)?;

    let instruction = InstructionView {
        program_id: &DELEGATION_PROGRAM_ID,
        accounts: unsafe {
            core::slice::from_raw_parts(
                account_metas.as_ptr() as *const InstructionAccount,
                num_accounts,
            )
        },
        data: &data,
    };

    let mut acc_infos: [&AccountView; MAX_ACCOUNTS] = [payer; MAX_ACCOUNTS];
    acc_infos[0] = payer;
    acc_infos[1] = pda_acc;
    acc_infos[2] = owner_program;
    acc_infos[3] = buffer_acc;
    acc_infos[4] = delegation_record;
    acc_infos[5] = delegation_metadata;
    acc_infos[6] = system_program;
    let mut j = 0;
    while j < action_signer_accounts.len() {
        acc_infos[7 + j] = action_signer_accounts[j];
        j += 1;
    }

    invoke_signed_with_bounds::<MAX_ACCOUNTS>(
        &instruction,
        &acc_infos[..num_accounts],
        &[signer_seeds],
    )?;
    Ok(())
}

pub fn create_schedule_commit_ix<'a>(
    payer: &'a AccountView,
    account_views: &'a [AccountView],
    magic_context: &'a AccountView,
    magic_program: &'a AccountView,
    magic_fee_vault: Option<&'a AccountView>,
    allow_undelegation: bool,
    account_metas: &'a mut [MaybeUninit<InstructionAccount<'a>>],
) -> Result<InstructionView<'a, 'a, 'a, 'a>, ProgramError> {
    let num_prefix_accounts = if magic_fee_vault.is_some() { 3 } else { 2 };
    let num_accounts = num_prefix_accounts + account_views.len();

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
        // payer is signer, may or may not be writable
        account_metas
            .get_unchecked_mut(0)
            .write(InstructionAccount::new(
                payer.address(),
                payer.is_writable(),
                true,
            ));

        // magic_context is writable, not signer
        account_metas
            .get_unchecked_mut(1)
            .write(InstructionAccount::writable(magic_context.address()));

        // optional fee vault: writable, not signer
        if let Some(vault) = magic_fee_vault {
            account_metas
                .get_unchecked_mut(2)
                .write(InstructionAccount::writable(vault.address()));
        }

        for i in 0..account_views.len() {
            let a = account_views.get_unchecked(i);
            account_metas
                .get_unchecked_mut(num_prefix_accounts + i)
                .write(InstructionAccount::new(
                    a.address(),
                    a.is_writable(),
                    a.is_signer(),
                ));
        }
    }

    let ix = InstructionView {
        program_id: magic_program.address(),
        accounts: unsafe {
            core::slice::from_raw_parts(
                account_metas.as_ptr() as *const InstructionAccount,
                num_accounts,
            )
        },
        data: instruction_data,
    };

    Ok(ix)
}
