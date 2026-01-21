use core::mem::MaybeUninit;
use pinocchio::cpi::{invoke, invoke_signed, Signer, MAX_CPI_ACCOUNTS};
use pinocchio::instruction::{InstructionAccount, InstructionView};
use pinocchio::{error::ProgramError, AccountView, Address, ProgramResult};

/// Delegate permission to ephemeral rollups.
pub fn delegate_permission(
    accounts: &[&AccountView],
    permission_program: &Address,
    authority_is_signer: bool,
    permissioned_account_is_signer: bool,
    signer_seeds: Option<Signer<'_, '_>>,
) -> ProgramResult {
    if accounts.len() < 10 || accounts.len() > 11 {
        return Err(ProgramError::NotEnoughAccountKeys);
    }

    let payer = accounts[0];
    let authority = accounts[1];
    let permissioned_account = accounts[2];
    let permission = accounts[3];
    let system_program = accounts[4];
    let owner_program = accounts[5];
    let delegation_buffer = accounts[6];
    let delegation_record = accounts[7];
    let delegation_metadata = accounts[8];
    let delegation_program = accounts[9];
    let validator = if accounts.len() == 11 {
        Some(accounts[10])
    } else {
        None
    };

    if !authority_is_signer && !permissioned_account_is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }

    const UNINIT_ACCOUNT: MaybeUninit<InstructionAccount> =
        MaybeUninit::<InstructionAccount>::uninit();
    let mut account_metas = [UNINIT_ACCOUNT; MAX_CPI_ACCOUNTS];

    let num_accounts = if validator.is_some() { 11 } else { 10 };

    unsafe {
        account_metas
            .get_unchecked_mut(0)
            .write(InstructionAccount::writable_signer(payer.address()));

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
        account_metas
            .get_unchecked_mut(4)
            .write(InstructionAccount::readonly(system_program.address()));
        account_metas
            .get_unchecked_mut(5)
            .write(InstructionAccount::readonly(owner_program.address()));
        account_metas
            .get_unchecked_mut(6)
            .write(InstructionAccount::writable(delegation_buffer.address()));
        account_metas
            .get_unchecked_mut(7)
            .write(InstructionAccount::writable(delegation_record.address()));
        account_metas
            .get_unchecked_mut(8)
            .write(InstructionAccount::writable(delegation_metadata.address()));
        account_metas
            .get_unchecked_mut(9)
            .write(InstructionAccount::readonly(delegation_program.address()));

        if let Some(validator_acc) = validator {
            account_metas
                .get_unchecked_mut(10)
                .write(InstructionAccount::readonly(validator_acc.address()));
        }
    }

    // Prepare instruction data with discriminator only (no args)
    let data = 3u64.to_le_bytes(); // DelegatePermission discriminator

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

    if let Some(validator_acc) = validator {
        let acc_infos: [&AccountView; 11] = [
            payer,
            authority,
            permissioned_account,
            permission,
            system_program,
            owner_program,
            delegation_buffer,
            delegation_record,
            delegation_metadata,
            delegation_program,
            validator_acc,
        ];
        if let Some(seeds) = signer_seeds {
            invoke_signed(&instruction, &acc_infos, &[seeds])?;
        } else {
            invoke(&instruction, &acc_infos)?;
        }
    } else {
        let acc_infos: [&AccountView; 10] = [
            payer,
            authority,
            permissioned_account,
            permission,
            system_program,
            owner_program,
            delegation_buffer,
            delegation_record,
            delegation_metadata,
            delegation_program,
        ];
        if let Some(seeds) = signer_seeds {
            invoke_signed(&instruction, &acc_infos, &[seeds])?;
        } else {
            invoke(&instruction, &acc_infos)?;
        }
    }

    Ok(())
}

pub struct DelegatePermissionCpiBuilder<'a> {
    payer: &'a AccountView,
    authority: &'a AccountView,
    permissioned_account: &'a AccountView,
    permission: &'a AccountView,
    system_program: &'a AccountView,
    owner_program: &'a AccountView,
    delegation_buffer: &'a AccountView,
    delegation_record: &'a AccountView,
    delegation_metadata: &'a AccountView,
    delegation_program: &'a AccountView,
    validator: Option<&'a AccountView>,
    permission_program: &'a Address,
    authority_is_signer: bool,
    permissioned_account_is_signer: bool,
    signer_seeds: Option<Signer<'a, 'a>>,
}

impl<'a> DelegatePermissionCpiBuilder<'a> {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        payer: &'a AccountView,
        authority: &'a AccountView,
        permissioned_account: &'a AccountView,
        permission: &'a AccountView,
        system_program: &'a AccountView,
        owner_program: &'a AccountView,
        delegation_buffer: &'a AccountView,
        delegation_record: &'a AccountView,
        delegation_metadata: &'a AccountView,
        delegation_program: &'a AccountView,
        validator: &'a AccountView,
        permission_program: &'a Address,
    ) -> Self {
        Self {
            payer,
            authority,
            permissioned_account,
            permission,
            system_program,
            owner_program,
            delegation_buffer,
            delegation_record,
            delegation_metadata,
            delegation_program,
            validator: Some(validator),
            permission_program,
            authority_is_signer: true,
            permissioned_account_is_signer: true,
            signer_seeds: None,
        }
    }

    pub fn authority_is_signer(mut self, authority_is_signer: bool) -> Self {
        self.authority_is_signer = authority_is_signer;
        self
    }

    pub fn permissioned_account_is_signer(mut self, permissioned_account_is_signer: bool) -> Self {
        self.permissioned_account_is_signer = permissioned_account_is_signer;
        self
    }

    pub fn signer_seeds(mut self, signer_seeds: Signer<'a, 'a>) -> Self {
        self.signer_seeds = Some(signer_seeds);
        self
    }

    pub fn invoke(self) -> ProgramResult {
        let accounts = [
            self.payer,
            self.authority,
            self.permissioned_account,
            self.permission,
            self.system_program,
            self.owner_program,
            self.delegation_buffer,
            self.delegation_record,
            self.delegation_metadata,
            self.delegation_program,
        ];
        if let Some(validator) = self.validator {
            let mut all_accounts = [self.payer; 11];
            all_accounts[..10].copy_from_slice(&accounts);
            all_accounts[10] = validator;
            delegate_permission(
                &all_accounts,
                self.permission_program,
                self.authority_is_signer,
                self.permissioned_account_is_signer,
                self.signer_seeds,
            )
        } else {
            delegate_permission(
                &accounts,
                self.permission_program,
                self.authority_is_signer,
                self.permissioned_account_is_signer,
                self.signer_seeds,
            )
        }
    }
}
