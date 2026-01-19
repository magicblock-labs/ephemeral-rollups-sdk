/// Delegate permission instruction builder
use crate::access_control::pinocchio::instructions::DELEGATE_PERMISSION_DISCRIMINATOR;
use crate::access_control::pinocchio::pda::{
    permission_pda_from_permissioned_account, permission_program_id,
};
use pinocchio::{instruction::InstructionAccount, error::ProgramError, Address, AccountView, cpi::invoke_with_slice};
use core::mem::MaybeUninit;

const MAX_INSTRUCTION_DATA: usize = 8; // Just discriminator

/// Instruction builder for `DelegatePermission`.
///
/// ### Accounts:
///   0. `[writable, signer]` payer
///   1. `[signer?]` authority - Either this or permissioned_account must be a signer
///   2. `[signer?]` permissioned_account - Either this or authority must be a signer
///   3. `[writable]` permission (auto-derived from permissioned_account)
///   4. `[]` system_program
///   5. `[]` owner_program
///   6. `[writable]` delegation_buffer
///   7. `[writable]` delegation_record
///   8. `[writable]` delegation_metadata
///   9. `[]` delegation_program
///   10. `[]` validator
#[derive(Clone, Debug, Default)]
pub struct DelegatePermissionBuilder {
    payer: Option<Address>,
    authority: Option<(Address, bool)>,
    permissioned_account: Option<(Address, bool)>,
    system_program: Option<Address>,
    owner_program: Option<Address>,
    delegation_buffer: Option<Address>,
    delegation_record: Option<Address>,
    delegation_metadata: Option<Address>,
    delegation_program: Option<Address>,
    validator: Option<Address>,
}

impl DelegatePermissionBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    #[inline(always)]
    pub fn payer(&mut self, payer: Address) -> &mut Self {
        self.payer = Some(payer);
        self
    }

    #[inline(always)]
    pub fn authority(&mut self, authority: Address, as_signer: bool) -> &mut Self {
        self.authority = Some((authority, as_signer));
        self
    }

    #[inline(always)]
    pub fn permissioned_account(&mut self, permissioned_account: Address, as_signer: bool) -> &mut Self {
        self.permissioned_account = Some((permissioned_account, as_signer));
        self
    }

    #[inline(always)]
    pub fn system_program(&mut self, system_program: Address) -> &mut Self {
        self.system_program = Some(system_program);
        self
    }

    #[inline(always)]
    pub fn owner_program(&mut self, owner_program: Address) -> &mut Self {
        self.owner_program = Some(owner_program);
        self
    }

    #[inline(always)]
    pub fn delegation_buffer(&mut self, delegation_buffer: Address) -> &mut Self {
        self.delegation_buffer = Some(delegation_buffer);
        self
    }

    #[inline(always)]
    pub fn delegation_record(&mut self, delegation_record: Address) -> &mut Self {
        self.delegation_record = Some(delegation_record);
        self
    }

    #[inline(always)]
    pub fn delegation_metadata(&mut self, delegation_metadata: Address) -> &mut Self {
        self.delegation_metadata = Some(delegation_metadata);
        self
    }

    #[inline(always)]
    pub fn delegation_program(&mut self, delegation_program: Address) -> &mut Self {
        self.delegation_program = Some(delegation_program);
        self
    }

    #[inline(always)]
    pub fn validator(&mut self, validator: Address) -> &mut Self {
        self.validator = Some(validator);
        self
    }

    pub fn instruction<'a>(
        &self,
        data_buf: &'a mut [u8],
    ) -> Result<(
        [MaybeUninit<InstructionAccount>; 11],
        usize,
        Address,
        &'a [u8],
    ), ProgramError> {
        let payer = self.payer.as_ref().ok_or(ProgramError::InvalidArgument)?;
        let authority = self.authority.as_ref().ok_or(ProgramError::InvalidArgument)?;
        let permissioned_account = self
            .permissioned_account
            .as_ref()
            .ok_or(ProgramError::InvalidArgument)?;
        let system_program = self
            .system_program
            .as_ref()
            .ok_or(ProgramError::InvalidArgument)?;
        let owner_program = self
            .owner_program
            .as_ref()
            .ok_or(ProgramError::InvalidArgument)?;
        let delegation_buffer = self
            .delegation_buffer
            .as_ref()
            .ok_or(ProgramError::InvalidArgument)?;
        let delegation_record = self
            .delegation_record
            .as_ref()
            .ok_or(ProgramError::InvalidArgument)?;
        let delegation_metadata = self
            .delegation_metadata
            .as_ref()
            .ok_or(ProgramError::InvalidArgument)?;
        let delegation_program = self
            .delegation_program
            .as_ref()
            .ok_or(ProgramError::InvalidArgument)?;
        let validator = self.validator.as_ref().ok_or(ProgramError::InvalidArgument)?;

        delegate_permission_instruction_impl(
            payer,
            (&authority.0, authority.1),
            (&permissioned_account.0, permissioned_account.1),
            system_program,
            owner_program,
            delegation_buffer,
            delegation_record,
            delegation_metadata,
            delegation_program,
            validator,
            data_buf,
        )
    }
}

/// Build a delegate_permission instruction
///
/// ### Accounts:
///   0. `[writable, signer]` payer
///   1. `[signer?]` authority - Either this or permissioned_account must be a signer
///   2. `[signer?]` permissioned_account - Either this or authority must be a signer
///   3. `[writable]` permission (auto-derived from permissioned_account)
///   4. `[]` system_program
///   5. `[]` owner_program
///   6. `[writable]` delegation_buffer
///   7. `[writable]` delegation_record
///   8. `[writable]` delegation_metadata
///   9. `[]` delegation_program
///   10. `[]` validator
fn delegate_permission_instruction_impl<'a>(
    payer: &Address,
    authority: (&Address, bool),
    permissioned_account: (&Address, bool),
    system_program: &Address,
    owner_program: &Address,
    delegation_buffer: &Address,
    delegation_record: &Address,
    delegation_metadata: &Address,
    delegation_program: &Address,
    validator: &Address,
    data_buf: &'a mut [u8],
) -> Result<(
    [MaybeUninit<InstructionAccount>; 11],
    usize,
    Address,
    &'a [u8],
), ProgramError> {
    let permission = permission_pda_from_permissioned_account(permissioned_account.0).0;
    if data_buf.len() < MAX_INSTRUCTION_DATA {
        return Err(ProgramError::InvalidArgument);
    }

    const UNINIT_META: MaybeUninit<InstructionAccount> = MaybeUninit::<InstructionAccount>::uninit();
    let mut metas = [UNINIT_META; 11];

    unsafe {
        metas[0].write(InstructionAccount {
            address: payer,
            is_writable: true,
            is_signer: true,
        });
        metas[1].write(InstructionAccount {
            address: authority.0,
            is_writable: false,
            is_signer: authority.1,
        });
        metas[2].write(InstructionAccount {
            address: permissioned_account.0,
            is_writable: false,
            is_signer: permissioned_account.1,
        });
        metas[3].write(InstructionAccount {
            address: &permission,
            is_writable: true,
            is_signer: false,
        });
        metas[4].write(InstructionAccount {
            address: system_program,
            is_writable: false,
            is_signer: false,
        });
        metas[5].write(InstructionAccount {
            address: owner_program,
            is_writable: false,
            is_signer: false,
        });
        metas[6].write(InstructionAccount {
            address: delegation_buffer,
            is_writable: true,
            is_signer: false,
        });
        metas[7].write(InstructionAccount {
            address: delegation_record,
            is_writable: true,
            is_signer: false,
        });
        metas[8].write(InstructionAccount {
            address: delegation_metadata,
            is_writable: true,
            is_signer: false,
        });
        metas[9].write(InstructionAccount {
            address: delegation_program,
            is_writable: false,
            is_signer: false,
        });
        metas[10].write(InstructionAccount {
            address: validator,
            is_writable: false,
            is_signer: false,
        });
    }

    // Serialize discriminator
    let discriminator_bytes = DELEGATE_PERMISSION_DISCRIMINATOR.to_le_bytes();
    data_buf[0..8].copy_from_slice(&discriminator_bytes);

    Ok((metas, 11, permission_program_id(), &data_buf[..8]))
}

/// `delegate_permission` CPI accounts using AccountView.
pub struct DelegatePermissionCpiAccountsAccountView<'a> {
    pub payer: &'a AccountView,
    pub authority: (&'a AccountView, bool),
    pub permissioned_account: (&'a AccountView, bool),
    pub system_program: &'a AccountView,
    pub owner_program: &'a AccountView,
    pub delegation_buffer: &'a AccountView,
    pub delegation_record: &'a AccountView,
    pub delegation_metadata: &'a AccountView,
    pub delegation_program: &'a AccountView,
    pub validator: &'a AccountView,
}

/// `delegate_permission` CPI instruction using AccountView.
pub struct DelegatePermissionCpiAccountView<'a> {
    /// The program to invoke.
    pub __program: &'a AccountView,
    pub payer: &'a AccountView,
    pub authority: (&'a AccountView, bool),
    pub permissioned_account: (&'a AccountView, bool),
    pub system_program: &'a AccountView,
    pub owner_program: &'a AccountView,
    pub delegation_buffer: &'a AccountView,
    pub delegation_record: &'a AccountView,
    pub delegation_metadata: &'a AccountView,
    pub delegation_program: &'a AccountView,
    pub validator: &'a AccountView,
}

impl<'a> DelegatePermissionCpiAccountView<'a> {
    pub fn new(
        program: &'a AccountView,
        accounts: DelegatePermissionCpiAccountsAccountView<'a>,
    ) -> Self {
        Self {
            __program: program,
            payer: accounts.payer,
            authority: accounts.authority,
            permissioned_account: accounts.permissioned_account,
            system_program: accounts.system_program,
            owner_program: accounts.owner_program,
            delegation_buffer: accounts.delegation_buffer,
            delegation_record: accounts.delegation_record,
            delegation_metadata: accounts.delegation_metadata,
            delegation_program: accounts.delegation_program,
            validator: accounts.validator,
        }
    }

    #[inline(always)]
    pub fn invoke(&self) -> Result<(), ProgramError> {
        self.invoke_with_remaining_accounts(&[])
    }

    #[inline(always)]
    pub fn invoke_with_remaining_accounts(
        &self,
        remaining_accounts: &[(&'a AccountView, bool, bool)],
    ) -> Result<(), ProgramError> {
        let mut data_buf = [0u8; MAX_INSTRUCTION_DATA];
        let (metas, num_accounts, program_id, data) = delegate_permission_instruction_impl(
            &self.payer.address,
            (&self.authority.0.address, self.authority.1),
            (&self.permissioned_account.0.address, self.permissioned_account.1),
            &self.system_program.address,
            &self.owner_program.address,
            &self.delegation_buffer.address,
            &self.delegation_record.address,
            &self.delegation_metadata.address,
            &self.delegation_program.address,
            &self.validator.address,
            &mut data_buf,
        )?;

        let mut all_accounts: [&AccountView; 16] = [self.__program; 16];
        all_accounts[0] = self.payer;
        all_accounts[1] = self.authority.0;
        all_accounts[2] = self.permissioned_account.0;
        all_accounts[3] = self.system_program;
        all_accounts[4] = self.owner_program;
        all_accounts[5] = self.delegation_buffer;
        all_accounts[6] = self.delegation_record;
        all_accounts[7] = self.delegation_metadata;
        all_accounts[8] = self.delegation_program;
        all_accounts[9] = self.validator;

        let mut account_count = num_accounts;
        for (account, _, _) in remaining_accounts.iter() {
            if account_count >= all_accounts.len() {
                return Err(ProgramError::InvalidArgument);
            }
            all_accounts[account_count] = account;
            account_count += 1;
        }

        invoke_with_slice(&(metas, program_id, data), &all_accounts[..account_count])
    }
}

/// Instruction builder for `DelegatePermission` via CPI with AccountView.
///
/// ### Accounts:
///   0. `[writable, signer]` payer
///   1. `[signer?]` authority - Either this or permissioned_account must be a signer
///   2. `[signer?]` permissioned_account - Either this or authority must be a signer
///   3. `[writable]` permission (auto-derived from permissioned_account)
///   4. `[]` system_program
///   5. `[]` owner_program
///   6. `[writable]` delegation_buffer
///   7. `[writable]` delegation_record
///   8. `[writable]` delegation_metadata
///   9. `[]` delegation_program
///   10. `[]` validator
#[derive(Clone, Debug)]
pub struct DelegatePermissionCpiBuilderAccountView<'a> {
    instruction: Box<DelegatePermissionCpiBuilderInstructionAccountView<'a>>,
}

impl<'a> DelegatePermissionCpiBuilderAccountView<'a> {
    pub fn new(program: &'a AccountView) -> Self {
        let instruction = Box::new(DelegatePermissionCpiBuilderInstructionAccountView {
            __program: program,
            payer: None,
            authority: None,
            permissioned_account: None,
            system_program: None,
            owner_program: None,
            delegation_buffer: None,
            delegation_record: None,
            delegation_metadata: None,
            delegation_program: None,
            validator: None,
            __remaining_accounts: Vec::new(),
        });
        Self { instruction }
    }

    #[inline(always)]
    pub fn payer(&mut self, payer: &'a AccountView) -> &mut Self {
        self.instruction.payer = Some(payer);
        self
    }

    #[inline(always)]
    pub fn authority(&mut self, authority: &'a AccountView, as_signer: bool) -> &mut Self {
        self.instruction.authority = Some((authority, as_signer));
        self
    }

    #[inline(always)]
    pub fn permissioned_account(
        &mut self,
        permissioned_account: &'a AccountView,
        as_signer: bool,
    ) -> &mut Self {
        self.instruction.permissioned_account = Some((permissioned_account, as_signer));
        self
    }

    #[inline(always)]
    pub fn system_program(&mut self, system_program: &'a AccountView) -> &mut Self {
        self.instruction.system_program = Some(system_program);
        self
    }

    #[inline(always)]
    pub fn owner_program(&mut self, owner_program: &'a AccountView) -> &mut Self {
        self.instruction.owner_program = Some(owner_program);
        self
    }

    #[inline(always)]
    pub fn delegation_buffer(&mut self, delegation_buffer: &'a AccountView) -> &mut Self {
        self.instruction.delegation_buffer = Some(delegation_buffer);
        self
    }

    #[inline(always)]
    pub fn delegation_record(&mut self, delegation_record: &'a AccountView) -> &mut Self {
        self.instruction.delegation_record = Some(delegation_record);
        self
    }

    #[inline(always)]
    pub fn delegation_metadata(&mut self, delegation_metadata: &'a AccountView) -> &mut Self {
        self.instruction.delegation_metadata = Some(delegation_metadata);
        self
    }

    #[inline(always)]
    pub fn delegation_program(&mut self, delegation_program: &'a AccountView) -> &mut Self {
        self.instruction.delegation_program = Some(delegation_program);
        self
    }

    #[inline(always)]
    pub fn validator(&mut self, validator: &'a AccountView) -> &mut Self {
        self.instruction.validator = Some(validator);
        self
    }

    /// Add an additional account to the instruction.
    #[inline(always)]
    pub fn add_remaining_account(
        &mut self,
        account: &'a AccountView,
        is_writable: bool,
        is_signer: bool,
    ) -> &mut Self {
        self.instruction
            .__remaining_accounts
            .push((account, is_writable, is_signer));
        self
    }

    /// Add additional accounts to the instruction.
    #[inline(always)]
    pub fn add_remaining_accounts(
        &mut self,
        accounts: &[(&'a AccountView, bool, bool)],
    ) -> &mut Self {
        self.instruction
            .__remaining_accounts
            .extend_from_slice(accounts);
        self
    }

    #[inline(always)]
    pub fn invoke(&self) -> Result<(), ProgramError> {
        let instruction = DelegatePermissionCpiAccountView {
            __program: self.instruction.__program,
            payer: self.instruction.payer.expect("payer is not set"),
            authority: self.instruction.authority.expect("authority is not set"),
            permissioned_account: self
                .instruction
                .permissioned_account
                .expect("permissioned_account is not set"),
            system_program: self
                .instruction
                .system_program
                .expect("system_program is not set"),
            owner_program: self
                .instruction
                .owner_program
                .expect("owner_program is not set"),
            delegation_buffer: self
                .instruction
                .delegation_buffer
                .expect("delegation_buffer is not set"),
            delegation_record: self
                .instruction
                .delegation_record
                .expect("delegation_record is not set"),
            delegation_metadata: self
                .instruction
                .delegation_metadata
                .expect("delegation_metadata is not set"),
            delegation_program: self
                .instruction
                .delegation_program
                .expect("delegation_program is not set"),
            validator: self.instruction.validator.expect("validator is not set"),
        };
        instruction.invoke_with_remaining_accounts(&self.instruction.__remaining_accounts)
    }
}

#[derive(Clone, Debug)]
struct DelegatePermissionCpiBuilderInstructionAccountView<'a> {
    __program: &'a AccountView,
    payer: Option<&'a AccountView>,
    authority: Option<(&'a AccountView, bool)>,
    permissioned_account: Option<(&'a AccountView, bool)>,
    system_program: Option<&'a AccountView>,
    owner_program: Option<&'a AccountView>,
    delegation_buffer: Option<&'a AccountView>,
    delegation_record: Option<&'a AccountView>,
    delegation_metadata: Option<&'a AccountView>,
    delegation_program: Option<&'a AccountView>,
    validator: Option<&'a AccountView>,
    /// Additional instruction accounts `(AccountView, is_writable, is_signer)`.
    __remaining_accounts: Vec<(&'a AccountView, bool, bool)>,
}
