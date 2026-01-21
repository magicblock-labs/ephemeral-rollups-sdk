use core::mem::MaybeUninit;
use pinocchio::{
    cpi::{invoke, invoke_signed, Signer, MAX_CPI_ACCOUNTS},
    error::ProgramError,
    instruction::{InstructionAccount, InstructionView},
    AccountView, Address,
};

use crate::acl::consts::{
    CLOSE_PERMISSION_DISCRIMINATOR, CREATE_PERMISSION_DISCRIMINATOR,
    UPDATE_PERMISSION_DISCRIMINATOR,
};
use crate::acl::types::{MembersArgs, MAX_MEMBERS_ARGS_SIZE};

pub fn cpi_create_permission(
    permissioned_account: &AccountView,
    permission: &AccountView,
    payer: &AccountView,
    system_program: &AccountView,
    permission_program: &Address,
    args: MembersArgs,
    signer_seeds: Option<Signer<'_, '_>>,
) -> Result<(), ProgramError> {
    const UNINIT_ACCOUNT: MaybeUninit<InstructionAccount> =
        MaybeUninit::<InstructionAccount>::uninit();
    let mut account_metas = [UNINIT_ACCOUNT; MAX_CPI_ACCOUNTS];

    let num_accounts = 4;

    unsafe {
        account_metas
            .get_unchecked_mut(0)
            .write(InstructionAccount::readonly_signer(
                permissioned_account.address(),
            ));
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

    // Prepare instruction data with 8-byte discriminator prefix followed by serialized args
    // Calculate exact size needed: 8 bytes for discriminator + actual args size
    let args_size = args.serialized_size();
    let total_size = 8 + args_size;
    let mut data = [0u8; 8 + MAX_MEMBERS_ARGS_SIZE];

    // Write discriminator (create as u64 in little-endian)
    data[0..8].copy_from_slice(&CREATE_PERMISSION_DISCRIMINATOR.to_le_bytes());

    // Serialize args into the slice after the discriminator
    args.try_to_slice(&mut data[8..])?;
    let data_slice = &data[..total_size];

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

    let acc_infos: [&AccountView; 4] = [permissioned_account, permission, payer, system_program];

    if let Some(seeds) = signer_seeds {
        invoke_signed(&instruction, &acc_infos, &[seeds])?;
    } else {
        invoke(&instruction, &acc_infos)?;
    }
    Ok(())
}

#[allow(clippy::too_many_arguments)]
pub fn cpi_update_permission(
    authority: &AccountView,
    permissioned_account: &AccountView,
    permission: &AccountView,
    permission_program: &Address,
    authority_is_signer: bool,
    permissioned_account_is_signer: bool,
    args: MembersArgs,
    signer_seeds: Option<Signer<'_, '_>>,
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
                .write(InstructionAccount::writable_signer(
                    permissioned_account.address(),
                ));
        } else {
            account_metas
                .get_unchecked_mut(1)
                .write(InstructionAccount::readonly(permissioned_account.address()));
        }

        account_metas
            .get_unchecked_mut(2)
            .write(InstructionAccount::writable(permission.address()));
    }

    // Prepare instruction data with 8-byte discriminator prefix followed by serialized args
    // Calculate exact size needed: 8 bytes for discriminator + actual args size
    let args_size = args.serialized_size();
    let total_size = 8 + args_size;
    let mut data = [0u8; 8 + MAX_MEMBERS_ARGS_SIZE];

    // Write discriminator (update as u64 in little-endian)
    data[0..8].copy_from_slice(&UPDATE_PERMISSION_DISCRIMINATOR.to_le_bytes());

    // Serialize args into the slice after the discriminator
    args.try_to_slice(&mut data[8..])?;
    let data_slice = &data[..total_size];

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

    let acc_infos: [&AccountView; 3] = [authority, permissioned_account, permission];

    if let Some(seeds) = signer_seeds {
        invoke_signed(&instruction, &acc_infos, &[seeds])?;
    } else {
        invoke(&instruction, &acc_infos)?;
    }
    Ok(())
}

#[allow(clippy::too_many_arguments)]
pub fn cpi_close_permission(
    payer: &AccountView,
    authority: &AccountView,
    permissioned_account: &AccountView,
    permission: &AccountView,
    permission_program: &Address,
    authority_is_signer: bool,
    permissioned_account_is_signer: bool,
    signer_seeds: Option<Signer<'_, '_>>,
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
                .write(InstructionAccount::writable_signer(
                    permissioned_account.address(),
                ));
        } else {
            account_metas
                .get_unchecked_mut(2)
                .write(InstructionAccount::readonly(permissioned_account.address()));
        }

        account_metas
            .get_unchecked_mut(3)
            .write(InstructionAccount::writable(permission.address()));
    }

    // Prepare instruction data with discriminator only (no args)
    let data = CLOSE_PERMISSION_DISCRIMINATOR.to_le_bytes();

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

    let acc_infos: [&AccountView; 4] = [payer, authority, permissioned_account, permission];

    if let Some(seeds) = signer_seeds {
        invoke_signed(&instruction, &acc_infos, &[seeds])?;
    } else {
        invoke(&instruction, &acc_infos)?;
    }
    Ok(())
}
