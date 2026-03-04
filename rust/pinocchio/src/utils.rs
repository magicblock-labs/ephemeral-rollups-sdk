use crate::{
    consts::{BUFFER, DELEGATION_PROGRAM_ID},
    pda::find_program_address,
    types::{
        DelegateAccountArgs, PostDelegationActions, MAX_DELEGATE_ACCOUNT_ARGS_SIZE,
        MAX_POST_DELEGATION_ACTIONS_SIZE,
    },
};
use core::mem::MaybeUninit;
use pinocchio::{
    address::MAX_SEEDS,
    cpi::{invoke_signed, Seed, Signer, MAX_CPI_ACCOUNTS},
    error::ProgramError,
    instruction::{InstructionAccount, InstructionView},
    AccountView, Address, ProgramResult,
};
use pinocchio_system::instructions::{Assign, CreateAccount};

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

/// Find the bump for a buffer PDA using the pinocchio PDA derivation.
fn find_buffer_pda_bump(pda_key: &[u8], owner_program: &Address) -> u8 {
    let (_, bump) = find_program_address(&[BUFFER, pda_key], owner_program);
    bump
}

pub fn fill_seeds<'a>(
    out: &'a mut [Seed<'a>; 16],
    seeds: &[&'a [u8]],
    bump_ref: &'a u8,
) -> &'a [Seed<'a>] {
    assert!(seeds.len() <= 15, "too many seeds (max 15 + bump = 16)");

    let bump_slice: &[u8] = core::slice::from_ref(bump_ref);

    let mut i = 0;
    while i < seeds.len() {
        out[i] = Seed::from(seeds[i]);
        i += 1;
    }
    out[i] = Seed::from(bump_slice);

    &out[..=i]
}

#[allow(clippy::too_many_arguments)]
pub fn cpi_delegate_prepare(
    payer: &AccountView,
    pda_acc: &AccountView,
    owner_program: &AccountView,
    buffer_acc: &AccountView,
    delegate_signer_seeds: &Signer<'_, '_>,
) -> ProgramResult {
    if !payer.is_signer() {
        return Err(ProgramError::MissingRequiredSignature);
    }

    // Buffer PDA seeds
    let pda_key_bytes: &[u8; 32] = pda_acc.address().as_array();

    // Find buffer PDA bump
    let buffer_pda_bump = find_buffer_pda_bump(pda_key_bytes.as_ref(), owner_program.address());

    // Buffer signer seeds
    let buffer_bump_slice = [buffer_pda_bump];
    let buffer_seed_binding = [
        Seed::from(BUFFER),
        Seed::from(pda_key_bytes.as_ref()),
        Seed::from(&buffer_bump_slice),
    ];
    let buffer_signer_seeds = Signer::from(&buffer_seed_binding);

    // Single data_len and rent lookup
    let data_len = pda_acc.data_len();

    // Create Buffer PDA
    CreateAccount {
        from: payer,
        to: buffer_acc,
        lamports: 0,
        space: data_len as u64,
        owner: owner_program.address(),
    }
    .invoke_signed(&[buffer_signer_seeds])?;

    // Copy delegated PDA -> buffer, then zero delegated PDA
    {
        let pda_ro = pda_acc.try_borrow()?;
        let mut buf_data = buffer_acc.try_borrow_mut()?;
        buf_data.copy_from_slice(&pda_ro);
    }
    {
        let mut pda_mut = pda_acc.try_borrow_mut()?;
        for b in pda_mut.iter_mut().take(data_len) {
            *b = 0;
        }
    }

    let current_owner = unsafe { pda_acc.owner() };
    if current_owner != &pinocchio_system::ID {
        unsafe { pda_acc.assign(&pinocchio_system::ID) };
    }
    let current_owner = unsafe { pda_acc.owner() };
    if current_owner != &DELEGATION_PROGRAM_ID {
        Assign {
            account: pda_acc,
            owner: &DELEGATION_PROGRAM_ID,
        }
        .invoke_signed(&[delegate_signer_seeds.clone()])?;
    }

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

    // Prepare instruction data with 8-byte discriminator prefix followed by serialized args
    let mut data = [0u8; 8 + MAX_DELEGATE_ACCOUNT_ARGS_SIZE];

    // Serialize args into the slice after the discriminator
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
    actions: PostDelegationActions,
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

    // Prepare instruction data with 8-byte discriminator prefix followed by serialized args
    let mut data = [0u8; 8 + MAX_DELEGATE_ACCOUNT_ARGS_SIZE + MAX_POST_DELEGATION_ACTIONS_SIZE];

    // Serialize args into the slice after the discriminator
    let args_len = {
        let args_slice = delegate_args.try_to_slice(&mut data[8..])?;
        args_slice.len()
    };
    let actions_len = {
        let actions_slice = actions.try_to_slice(&mut data[8 + args_len..])?;
        actions_slice.len()
    };
    let total_len = 8 + args_len + actions_len;
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

pub fn create_schedule_commit_ix<'a>(
    payer: &'a AccountView,
    account_views: &'a [AccountView],
    magic_context: &'a AccountView,
    magic_program: &'a AccountView,
    allow_undelegation: bool,
    account_metas: &'a mut [MaybeUninit<InstructionAccount<'a>>],
) -> Result<InstructionView<'a, 'a, 'a, 'a>, ProgramError> {
    let num_accounts = 2 + account_views.len();

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

        for i in 0..account_views.len() {
            let a = account_views.get_unchecked(i);
            account_metas
                .get_unchecked_mut(2 + i)
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
