use crate::{
    consts::DELEGATION_PROGRAM_ID,
    types::{DelegateAccountArgs, MembersArgs, MAX_DELEGATE_ACCOUNT_ARGS_SIZE, MAX_MEMBERS_ARGS_SIZE},
};
use core::mem::MaybeUninit;
use pinocchio::{
    address::MAX_SEEDS,
    cpi::{invoke, invoke_signed, Seed, Signer, MAX_CPI_ACCOUNTS},
    error::ProgramError,
    instruction::{InstructionAccount, InstructionView},
    AccountView, Address,
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

// Permission program CPI helpers
pub fn cpi_create_permission(
    permissioned_account: &AccountView,
    permission: &AccountView,
    payer: &AccountView,
    system_program: &AccountView,
    permission_program: &Address,
    args: MembersArgs,
) -> Result<(), ProgramError> {
    const UNINIT_ACCOUNT: MaybeUninit<InstructionAccount> =
        MaybeUninit::<InstructionAccount>::uninit();
    let mut account_metas = [UNINIT_ACCOUNT; MAX_CPI_ACCOUNTS];

    let num_accounts = 4;

    unsafe {
        account_metas
            .get_unchecked_mut(0)
            .write(InstructionAccount::readonly_signer(permissioned_account.address()));
        account_metas
            .get_unchecked_mut(1)
            .write(InstructionAccount::writable(permission.address()));
        account_metas
            .get_unchecked_mut(2)
            .write(InstructionAccount::writable_signer(payer.address()));
        account_metas
            .get_unchecked_mut(3)
            .write(InstructionAccount::readonly(system_program.address()));
    }

    let mut data = [0u8; 8 + MAX_MEMBERS_ARGS_SIZE];
    let args_slice = args.try_to_slice(&mut data[8..])?;
    let total_len = 8 + args_slice.len();
    let data_slice = &data[..total_len];

    let instruction = InstructionView {
        program_id: permission_program,
        accounts: unsafe {
            core::slice::from_raw_parts(
                account_metas.as_ptr() as *const InstructionAccount,
                num_accounts,
            )
        },
        data: data_slice,
    };

    let acc_infos: [&AccountView; 4] = [
        permissioned_account,
        permission,
        payer,
        system_program,
    ];

    invoke(&instruction, &acc_infos)?;
    Ok(())
}

pub fn cpi_update_permission(
    authority: &AccountView,
    permissioned_account: &AccountView,
    permission: &AccountView,
    permission_program: &Address,
    authority_is_signer: bool,
    permissioned_account_is_signer: bool,
    args: MembersArgs,
) -> Result<(), ProgramError> {
    const UNINIT_ACCOUNT: MaybeUninit<InstructionAccount> =
        MaybeUninit::<InstructionAccount>::uninit();
    let mut account_metas = [UNINIT_ACCOUNT; MAX_CPI_ACCOUNTS];

    let num_accounts = 3;

    unsafe {
        // authority can be signer or not
        if authority_is_signer {
            account_metas
                .get_unchecked_mut(0)
                .write(InstructionAccount::writable_signer(authority.address()));
        } else {
            account_metas
                .get_unchecked_mut(0)
                .write(InstructionAccount::readonly(authority.address()));
        }

        // permissioned_account can be signer or not
        if permissioned_account_is_signer {
            account_metas
                .get_unchecked_mut(1)
                .write(InstructionAccount::writable_signer(permissioned_account.address()));
        } else {
            account_metas
                .get_unchecked_mut(1)
                .write(InstructionAccount::readonly(permissioned_account.address()));
        }

        account_metas
            .get_unchecked_mut(2)
            .write(InstructionAccount::writable(permission.address()));
    }

    let mut data = [0u8; 8 + MAX_MEMBERS_ARGS_SIZE];
    let args_slice = args.try_to_slice(&mut data[8..])?;
    let total_len = 8 + args_slice.len();
    let data_slice = &data[..total_len];

    let instruction = InstructionView {
        program_id: permission_program,
        accounts: unsafe {
            core::slice::from_raw_parts(
                account_metas.as_ptr() as *const InstructionAccount,
                num_accounts,
            )
        },
        data: data_slice,
    };

    let acc_infos: [&AccountView; 3] = [
        authority,
        permissioned_account,
        permission,
    ];

    invoke(&instruction, &acc_infos)?;
    Ok(())
}

pub fn cpi_close_permission(
    payer: &AccountView,
    authority: &AccountView,
    permissioned_account: &AccountView,
    permission: &AccountView,
    permission_program: &Address,
    authority_is_signer: bool,
    permissioned_account_is_signer: bool,
) -> Result<(), ProgramError> {
    const UNINIT_ACCOUNT: MaybeUninit<InstructionAccount> =
        MaybeUninit::<InstructionAccount>::uninit();
    let mut account_metas = [UNINIT_ACCOUNT; MAX_CPI_ACCOUNTS];

    let num_accounts = 4;

    unsafe {
        account_metas
            .get_unchecked_mut(0)
            .write(InstructionAccount::writable(payer.address()));

        // authority can be signer or not
        if authority_is_signer {
            account_metas
                .get_unchecked_mut(1)
                .write(InstructionAccount::writable_signer(authority.address()));
        } else {
            account_metas
                .get_unchecked_mut(1)
                .write(InstructionAccount::readonly(authority.address()));
        }

        // permissioned_account can be signer or not
        if permissioned_account_is_signer {
            account_metas
                .get_unchecked_mut(2)
                .write(InstructionAccount::writable_signer(permissioned_account.address()));
        } else {
            account_metas
                .get_unchecked_mut(2)
                .write(InstructionAccount::readonly(permissioned_account.address()));
        }

        account_metas
            .get_unchecked_mut(3)
            .write(InstructionAccount::writable(permission.address()));
    }

    let data = [2u8; 8]; // ClosePermission discriminator
    
    let instruction = InstructionView {
        program_id: permission_program,
        accounts: unsafe {
            core::slice::from_raw_parts(
                account_metas.as_ptr() as *const InstructionAccount,
                num_accounts,
            )
        },
        data: &data,
    };

    let acc_infos: [&AccountView; 4] = [
        payer,
        authority,
        permissioned_account,
        permission,
    ];

    invoke(&instruction, &acc_infos)?;
    Ok(())
}
