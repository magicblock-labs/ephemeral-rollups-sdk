use pinocchio::{error::ProgramError, AccountView, Address, ProgramResult};
use core::mem::MaybeUninit;
use pinocchio::cpi::{invoke, invoke_signed, Signer, MAX_CPI_ACCOUNTS};
use pinocchio::instruction::InstructionAccount;
use pinocchio::instruction::InstructionView;

/// Commit permission state to ephemeral rollups.
pub fn commit_permission(
    accounts: &[&AccountView],
    permission_program: &Address,
    authority_is_signer: bool,
    permissioned_account_is_signer: bool,
    signer_seeds: Option<Signer<'_, '_>>,
) -> ProgramResult {
    let [authority, permissioned_account, permission, magic_program, magic_context] = accounts
    else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };

    if !authority_is_signer && !permissioned_account_is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }

    const UNINIT_ACCOUNT: MaybeUninit<InstructionAccount> =
        MaybeUninit::<InstructionAccount>::uninit();
    let mut account_metas = [UNINIT_ACCOUNT; MAX_CPI_ACCOUNTS];

    let num_accounts = 5;

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
                .write(InstructionAccount::writable(permissioned_account.address()));
        }

        account_metas
            .get_unchecked_mut(2)
            .write(InstructionAccount::writable(permission.address()));
        account_metas
            .get_unchecked_mut(3)
            .write(InstructionAccount::readonly(magic_program.address()));
        account_metas
            .get_unchecked_mut(4)
            .write(InstructionAccount::writable(magic_context.address()));
    }

    let data = [4u8; 8]; // CommitPermission discriminator

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

    let acc_infos: [&AccountView; 5] = [
        authority,
        permissioned_account,
        permission,
        magic_program,
        magic_context,
    ];

    if let Some(seeds) = signer_seeds {
        invoke_signed(&instruction, &acc_infos, &[seeds])?;
    } else {
        invoke(&instruction, &acc_infos)?;
    }
    Ok(())
}
