/// Commit permission instruction builder
use crate::access_control::pinocchio::instructions::COMMIT_PERMISSION_DISCRIMINATOR;
use crate::access_control::pinocchio::pda::permission_program_id;
use pinocchio::{instruction::InstructionAccount, error::ProgramError, Address, AccountView, cpi::invoke_with_slice};
use core::mem::MaybeUninit;

const MAX_INSTRUCTION_DATA: usize = 8;

/// Instruction builder for `CommitPermission`.
///
/// ### Accounts:
///   0. `[signer?]` authority - Either this or permissioned_account must be a signer
///   1. `[writable, signer?]` permissioned_account - Either this or authority must be a signer
///   2. `[writable]` permission
///   3. `[]` magic_program
///   4. `[writable]` magic_context
#[derive(Clone, Debug, Default)]
pub struct CommitPermissionBuilder {
    authority: Option<(Address, bool)>,
    permissioned_account: Option<(Address, bool)>,
    permission: Option<Address>,
    magic_program: Option<Address>,
    magic_context: Option<Address>,
}

impl CommitPermissionBuilder {
    pub fn new() -> Self {
        Self::default()
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
    pub fn permission(&mut self, permission: Address) -> &mut Self {
        self.permission = Some(permission);
        self
    }

    #[inline(always)]
    pub fn magic_program(&mut self, magic_program: Address) -> &mut Self {
        self.magic_program = Some(magic_program);
        self
    }

    #[inline(always)]
    pub fn magic_context(&mut self, magic_context: Address) -> &mut Self {
        self.magic_context = Some(magic_context);
        self
    }

    pub fn instruction<'a>(
        &self,
        data_buf: &'a mut [u8],
    ) -> Result<(
        [MaybeUninit<InstructionAccount>; 5],
        usize,
        Address,
        &'a [u8],
    ), ProgramError> {
        let authority = self.authority.as_ref().ok_or(ProgramError::InvalidArgument)?;
        let permissioned_account = self
            .permissioned_account
            .as_ref()
            .ok_or(ProgramError::InvalidArgument)?;
        let permission = self.permission.as_ref().ok_or(ProgramError::InvalidArgument)?;
        let magic_program = self
            .magic_program
            .as_ref()
            .ok_or(ProgramError::InvalidArgument)?;
        let magic_context = self
            .magic_context
            .as_ref()
            .ok_or(ProgramError::InvalidArgument)?;

        commit_permission_instruction_impl(
            (&authority.0, authority.1),
            (&permissioned_account.0, permissioned_account.1),
            permission,
            magic_program,
            magic_context,
            data_buf,
        )
    }
}

/// Build a commit_permission instruction
///
/// ### Accounts:
///   0. `[signer?]` authority - Either this or permissioned_account must be a signer
///   1. `[writable, signer?]` permissioned_account - Either this or authority must be a signer
///   2. `[writable]` permission
///   3. `[]` magic_program
///   4. `[writable]` magic_context
fn commit_permission_instruction_impl<'a>(
    authority: (&Address, bool),
    permissioned_account: (&Address, bool),
    permission: &Address,
    magic_program: &Address,
    magic_context: &Address,
    data_buf: &'a mut [u8],
) -> Result<(
    [MaybeUninit<InstructionAccount>; 5],
    usize,
    Address,
    &'a [u8],
), ProgramError> {
    if data_buf.len() < MAX_INSTRUCTION_DATA {
        return Err(ProgramError::InvalidArgument);
    }

    const UNINIT_META: MaybeUninit<InstructionAccount> = MaybeUninit::<InstructionAccount>::uninit();
    let mut metas = [UNINIT_META; 5];

    unsafe {
        metas[0].write(InstructionAccount {
            address: authority.0,
            is_writable: false,
            is_signer: authority.1,
        });
        metas[1].write(InstructionAccount {
            address: permissioned_account.0,
            is_writable: true,
            is_signer: permissioned_account.1,
        });
        metas[2].write(InstructionAccount {
            address: permission,
            is_writable: true,
            is_signer: false,
        });
        metas[3].write(InstructionAccount {
            address: magic_program,
            is_writable: false,
            is_signer: false,
        });
        metas[4].write(InstructionAccount {
            address: magic_context,
            is_writable: true,
            is_signer: false,
        });
    }

    // Serialize discriminator
    let discriminator_bytes = COMMIT_PERMISSION_DISCRIMINATOR.to_le_bytes();
    data_buf[0..8].copy_from_slice(&discriminator_bytes);

    Ok((metas, 5, permission_program_id(), &data_buf[..8]))
}

/// `commit_permission` CPI accounts using AccountView.
pub struct CommitPermissionCpiAccountsAccountView<'a> {
    pub authority: (&'a AccountView, bool),
    pub permissioned_account: (&'a AccountView, bool),
    pub permission: &'a AccountView,
    pub magic_program: &'a AccountView,
    pub magic_context: &'a AccountView,
}

/// `commit_permission` CPI instruction using AccountView.
pub struct CommitPermissionCpiAccountView<'a> {
    /// The program to invoke.
    pub __program: &'a AccountView,
    pub authority: (&'a AccountView, bool),
    pub permissioned_account: (&'a AccountView, bool),
    pub permission: &'a AccountView,
    pub magic_program: &'a AccountView,
    pub magic_context: &'a AccountView,
}

impl<'a> CommitPermissionCpiAccountView<'a> {
    pub fn new(
        program: &'a AccountView,
        accounts: CommitPermissionCpiAccountsAccountView<'a>,
    ) -> Self {
        Self {
            __program: program,
            authority: accounts.authority,
            permissioned_account: accounts.permissioned_account,
            permission: accounts.permission,
            magic_program: accounts.magic_program,
            magic_context: accounts.magic_context,
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
        let (metas, num_accounts, program_id, data) = commit_permission_instruction_impl(
            (&self.authority.0.address, self.authority.1),
            (&self.permissioned_account.0.address, self.permissioned_account.1),
            &self.permission.address,
            &self.magic_program.address,
            &self.magic_context.address,
            &mut data_buf,
        )?;

        let mut all_accounts: [&AccountView; 10] = [self.__program; 10];
        all_accounts[0] = self.authority.0;
        all_accounts[1] = self.permissioned_account.0;
        all_accounts[2] = self.permission;
        all_accounts[3] = self.magic_program;
        all_accounts[4] = self.magic_context;

        let mut account_count = num_accounts;
        for (i, (account, _, _)) in remaining_accounts.iter().enumerate() {
            if account_count >= all_accounts.len() {
                return Err(ProgramError::InvalidArgument);
            }
            all_accounts[account_count] = account;
            account_count += 1;
        }

        invoke_with_slice(&(metas, program_id, data), &all_accounts[..account_count])
    }
}

/// Instruction builder for `CommitPermission` via CPI with AccountView.
///
/// ### Accounts (magic_program and magic_context are auto-set):
///
///   0. `[signer?]` authority - Either this or permissioned_account must be a signer
///   1. `[writable, signer?]` permissioned_account - Either this or authority must be a signer
///   2. `[writable]` permission
///   3. `[]` magic_program
///   4. `[writable]` magic_context
#[derive(Clone, Debug)]
pub struct CommitPermissionCpiBuilderAccountView<'a> {
    instruction: Box<CommitPermissionCpiBuilderInstructionAccountView<'a>>,
}

impl<'a> CommitPermissionCpiBuilderAccountView<'a> {
    pub fn new(program: &'a AccountView) -> Self {
        let instruction = Box::new(CommitPermissionCpiBuilderInstructionAccountView {
            __program: program,
            authority: None,
            permissioned_account: None,
            permission: None,
            magic_program: None,
            magic_context: None,
            __remaining_accounts: Vec::new(),
        });
        Self { instruction }
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
    pub fn permission(&mut self, permission: &'a AccountView) -> &mut Self {
        self.instruction.permission = Some(permission);
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
    ///
    /// Each account is represented by a tuple of the `AccountView`, a `bool` indicating whether the account is writable or not,
    /// and a `bool` indicating whether the account is a signer or not.
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
    pub fn magic_program(&mut self, magic_program: &'a AccountView) -> &mut Self {
        self.instruction.magic_program = Some(magic_program);
        self
    }

    #[inline(always)]
    pub fn magic_context(&mut self, magic_context: &'a AccountView) -> &mut Self {
        self.instruction.magic_context = Some(magic_context);
        self
    }

    #[inline(always)]
    pub fn invoke(&self) -> Result<(), ProgramError> {
        let instruction = CommitPermissionCpiAccountView {
            __program: self.instruction.__program,
            authority: self.instruction.authority.expect("authority is not set"),
            permissioned_account: self
                .instruction
                .permissioned_account
                .expect("permissioned_account is not set"),
            permission: self.instruction.permission.expect("permission is not set"),
            magic_program: self
                .instruction
                .magic_program
                .expect("magic_program is not set"),
            magic_context: self
                .instruction
                .magic_context
                .expect("magic_context is not set"),
        };
        instruction.invoke_with_remaining_accounts(&self.instruction.__remaining_accounts)
    }
}

#[derive(Clone, Debug)]
struct CommitPermissionCpiBuilderInstructionAccountView<'a> {
    __program: &'a AccountView,
    authority: Option<(&'a AccountView, bool)>,
    permissioned_account: Option<(&'a AccountView, bool)>,
    permission: Option<&'a AccountView>,
    magic_program: Option<&'a AccountView>,
    magic_context: Option<&'a AccountView>,
    /// Additional instruction accounts `(AccountView, is_writable, is_signer)`.
    __remaining_accounts: Vec<(&'a AccountView, bool, bool)>,
}
