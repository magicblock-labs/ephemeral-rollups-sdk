/// Close permission instruction builder
use crate::access_control::pinocchio::instructions::CLOSE_PERMISSION_DISCRIMINATOR;
use crate::access_control::pinocchio::pda::permission_program_id;
use pinocchio::{instruction::InstructionAccount, error::ProgramError, Address, AccountView, cpi::invoke_with_slice};
use core::mem::MaybeUninit;

const MAX_INSTRUCTION_DATA: usize = 8;

/// Instruction builder for `ClosePermission`.
///
/// ### Accounts:
///   0. `[writable, signer]` payer
///   1. `[signer?]` authority - Either this or permissioned_account must be a signer
///   2. `[signer?]` permissioned_account - Either this or authority must be a signer
///   3. `[writable]` permission
#[derive(Clone, Debug, Default)]
pub struct ClosePermissionBuilder {
    payer: Option<Address>,
    authority: Option<(Address, bool)>,
    permissioned_account: Option<(Address, bool)>,
    permission: Option<Address>,
}

impl ClosePermissionBuilder {
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
    pub fn permission(&mut self, permission: Address) -> &mut Self {
        self.permission = Some(permission);
        self
    }

    pub fn instruction<'a>(
        &self,
        data_buf: &'a mut [u8],
    ) -> Result<(
        [MaybeUninit<InstructionAccount>; 4],
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
        let permission = self.permission.as_ref().ok_or(ProgramError::InvalidArgument)?;

        close_permission_instruction_impl(
            payer,
            (&authority.0, authority.1),
            (&permissioned_account.0, permissioned_account.1),
            permission,
            data_buf,
        )
    }
}

/// Build a close_permission instruction
///
/// ### Accounts:
///   0. `[writable, signer]` payer
///   1. `[signer?]` authority - Either this or permissioned_account must be a signer
///   2. `[signer?]` permissioned_account - Either this or authority must be a signer
///   3. `[writable]` permission
fn close_permission_instruction_impl<'a>(
    payer: &Address,
    authority: (&Address, bool),
    permissioned_account: (&Address, bool),
    permission: &Address,
    data_buf: &'a mut [u8],
) -> Result<(
    [MaybeUninit<InstructionAccount>; 4],
    usize,
    Address,
    &'a [u8],
), ProgramError> {
    if data_buf.len() < MAX_INSTRUCTION_DATA {
        return Err(ProgramError::InvalidArgument);
    }

    const UNINIT_META: MaybeUninit<InstructionAccount> = MaybeUninit::<InstructionAccount>::uninit();
    let mut metas = [UNINIT_META; 4];

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
            address: permission,
            is_writable: true,
            is_signer: false,
        });
    }

    // Serialize discriminator
    let discriminator_bytes = CLOSE_PERMISSION_DISCRIMINATOR.to_le_bytes();
    data_buf[0..8].copy_from_slice(&discriminator_bytes);

    Ok((metas, 4, permission_program_id(), &data_buf[..8]))
}

/// `close_permission` CPI accounts using AccountView.
pub struct ClosePermissionCpiAccountsAccountView<'a> {
    pub payer: &'a AccountView,
    pub authority: (&'a AccountView, bool),
    pub permissioned_account: (&'a AccountView, bool),
    pub permission: &'a AccountView,
}

/// `close_permission` CPI instruction using AccountView.
pub struct ClosePermissionCpiAccountView<'a> {
    /// The program to invoke.
    pub __program: &'a AccountView,
    pub payer: &'a AccountView,
    pub authority: (&'a AccountView, bool),
    pub permissioned_account: (&'a AccountView, bool),
    pub permission: &'a AccountView,
}

impl<'a> ClosePermissionCpiAccountView<'a> {
    pub fn new(
        program: &'a AccountView,
        accounts: ClosePermissionCpiAccountsAccountView<'a>,
    ) -> Self {
        Self {
            __program: program,
            payer: accounts.payer,
            authority: accounts.authority,
            permissioned_account: accounts.permissioned_account,
            permission: accounts.permission,
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
        let (metas, num_accounts, program_id, data) = close_permission_instruction_impl(
            &self.payer.address,
            (&self.authority.0.address, self.authority.1),
            (&self.permissioned_account.0.address, self.permissioned_account.1),
            &self.permission.address,
            &mut data_buf,
        )?;

        let mut all_accounts: [&AccountView; 8] = [self.__program; 8];
        all_accounts[0] = self.payer;
        all_accounts[1] = self.authority.0;
        all_accounts[2] = self.permissioned_account.0;
        all_accounts[3] = self.permission;

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

/// Instruction builder for `ClosePermission` via CPI with AccountView.
///
/// ### Accounts:
///
///   0. `[writable, signer]` payer
///   1. `[signer?]` authority - Either this or permissioned_account must be a signer
///   2. `[signer?]` permissioned_account - Either this or authority must be a signer
///   3. `[writable]` permission
#[derive(Clone, Debug)]
pub struct ClosePermissionCpiBuilderAccountView<'a> {
    instruction: Box<ClosePermissionCpiBuilderInstructionAccountView<'a>>,
}

impl<'a> ClosePermissionCpiBuilderAccountView<'a> {
    pub fn new(program: &'a AccountView) -> Self {
        let instruction = Box::new(ClosePermissionCpiBuilderInstructionAccountView {
            __program: program,
            payer: None,
            authority: None,
            permissioned_account: None,
            permission: None,
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
        let instruction = ClosePermissionCpiAccountView {
            __program: self.instruction.__program,
            payer: self.instruction.payer.expect("payer is not set"),
            authority: self.instruction.authority.expect("authority is not set"),
            permissioned_account: self
                .instruction
                .permissioned_account
                .expect("permissioned_account is not set"),
            permission: self.instruction.permission.expect("permission is not set"),
        };
        instruction.invoke_with_remaining_accounts(&self.instruction.__remaining_accounts)
    }
}

#[derive(Clone, Debug)]
struct ClosePermissionCpiBuilderInstructionAccountView<'a> {
    __program: &'a AccountView,
    payer: Option<&'a AccountView>,
    authority: Option<(&'a AccountView, bool)>,
    permissioned_account: Option<(&'a AccountView, bool)>,
    permission: Option<&'a AccountView>,
    /// Additional instruction accounts `(AccountView, is_writable, is_signer)`.
    __remaining_accounts: Vec<(&'a AccountView, bool, bool)>,
}
