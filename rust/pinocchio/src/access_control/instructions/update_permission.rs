/// Update permission instruction builder
use crate::access_control::pinocchio::instructions::UPDATE_PERMISSION_DISCRIMINATOR;
use crate::access_control::pinocchio::pda::permission_program_id;
use crate::access_control::pinocchio::structs::Member;
use pinocchio::{instruction::InstructionAccount, error::ProgramError, Address, AccountView, cpi::invoke_with_slice};
use core::mem::MaybeUninit;

const MAX_MEMBERS_SIZE: usize = 4 + (33 * 512);
const MAX_INSTRUCTION_DATA: usize = 8 + MAX_MEMBERS_SIZE;

/// Instruction builder for `UpdatePermission`.
///
/// ### Accounts:
///   0. `[signer?]` authority - Either this or permissioned_account must be a signer
///   1. `[signer?]` permissioned_account - Either this or authority must be a signer
///   2. `[writable]` permission
#[derive(Clone, Debug, Default)]
pub struct UpdatePermissionBuilder {
    authority: Option<(Address, bool)>,
    permissioned_account: Option<(Address, bool)>,
    permission: Option<Address>,
    members: Option<Vec<Member>>,
}

impl UpdatePermissionBuilder {
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
    pub fn members(&mut self, members: Option<Vec<Member>>) -> &mut Self {
        self.members = members;
        self
    }

    pub fn instruction<'a>(
        &self,
        data_buf: &'a mut [u8],
    ) -> Result<(
        [MaybeUninit<InstructionAccount>; 3],
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

        update_permission_instruction_impl(
            (&authority.0, authority.1),
            (&permissioned_account.0, permissioned_account.1),
            permission,
            self.members.as_deref(),
            data_buf,
        )
    }
}

/// Build an update_permission instruction
///
/// ### Accounts:
///   0. `[signer?]` authority - Either this or permissioned_account must be a signer
///   1. `[signer?]` permissioned_account - Either this or authority must be a signer
///   2. `[writable]` permission
///
/// Either `authority` or `permissioned_account` (or both) must be signers.
fn update_permission_instruction_impl<'a>(
    authority: (&Address, bool),
    permissioned_account: (&Address, bool),
    permission: &Address,
    members: Option<&[Member]>,
    data_buf: &'a mut [u8],
) -> Result<(
    [MaybeUninit<InstructionAccount>; 3],
    usize,
    Address,
    &'a [u8],
), ProgramError> {
    if data_buf.len() < MAX_INSTRUCTION_DATA {
        return Err(ProgramError::InvalidArgument);
    }

    // At least one account must be a signer
    if !authority.1 && !permissioned_account.1 {
        return Err(ProgramError::MissingRequiredSignature);
    }

    const UNINIT_META: MaybeUninit<InstructionAccount> = MaybeUninit::<InstructionAccount>::uninit();
    let mut metas = [UNINIT_META; 3];

    unsafe {
        metas[0].write(InstructionAccount {
            address: authority.0,
            is_writable: false,
            is_signer: authority.1,
        });
        metas[1].write(InstructionAccount {
            address: permissioned_account.0,
            is_writable: false,
            is_signer: permissioned_account.1,
        });
        metas[2].write(InstructionAccount {
            address: permission,
            is_writable: true,
            is_signer: false,
        });
    }

    // Serialize discriminator
    let discriminator_bytes = UPDATE_PERMISSION_DISCRIMINATOR.to_le_bytes();
    data_buf[0..8].copy_from_slice(&discriminator_bytes);

    // Serialize members
    let members_len = serialize_members(&mut data_buf[8..], members)?;
    let total_len = 8 + members_len;

    Ok((metas, 3, permission_program_id(), &data_buf[..total_len]))
}

fn serialize_members(buf: &mut [u8], members: Option<&[Member]>) -> Result<usize, ProgramError> {
    if buf.len() < 4 {
        return Err(ProgramError::InvalidArgument);
    }

    let member_count = members.map(|m| m.len()).unwrap_or(0);

    if 4 + member_count * 33 > buf.len() {
        return Err(ProgramError::InvalidArgument);
    }

    let count_bytes = (member_count as u32).to_le_bytes();
    buf[0..4].copy_from_slice(&count_bytes);

    let mut offset = 4;

    if let Some(members) = members {
        for member in members {
            buf[offset] = member.flags;
            offset += 1;
            buf[offset..offset + 32].copy_from_slice(member.pubkey.as_ref());
            offset += 32;
        }
    }

    Ok(offset)
}

/// `update_permission` CPI accounts using AccountView.
pub struct UpdatePermissionCpiAccountsAccountView<'a> {
    pub authority: (&'a AccountView, bool),
    pub permissioned_account: (&'a AccountView, bool),
    pub permission: &'a AccountView,
}

/// `update_permission` CPI instruction using AccountView.
pub struct UpdatePermissionCpiAccountView<'a> {
    /// The program to invoke.
    pub __program: &'a AccountView,
    pub authority: (&'a AccountView, bool),
    pub permissioned_account: (&'a AccountView, bool),
    pub permission: &'a AccountView,
}

impl<'a> UpdatePermissionCpiAccountView<'a> {
    pub fn new(
        program: &'a AccountView,
        accounts: UpdatePermissionCpiAccountsAccountView<'a>,
    ) -> Self {
        Self {
            __program: program,
            authority: accounts.authority,
            permissioned_account: accounts.permissioned_account,
            permission: accounts.permission,
        }
    }

    #[inline(always)]
    pub fn invoke(&self, members: Option<&[Member]>) -> Result<(), ProgramError> {
        self.invoke_with_remaining_accounts(members, &[])
    }

    #[inline(always)]
    pub fn invoke_with_remaining_accounts(
        &self,
        members: Option<&[Member]>,
        remaining_accounts: &[(&'a AccountView, bool, bool)],
    ) -> Result<(), ProgramError> {
        let mut data_buf = [0u8; MAX_INSTRUCTION_DATA];
        let (metas, num_accounts, program_id, data) = update_permission_instruction_impl(
            (&self.authority.0.address, self.authority.1),
            (&self.permissioned_account.0.address, self.permissioned_account.1),
            &self.permission.address,
            members,
            &mut data_buf,
        )?;

        let mut all_accounts: [&AccountView; 8] = [self.__program; 8];
        all_accounts[0] = self.authority.0;
        all_accounts[1] = self.permissioned_account.0;
        all_accounts[2] = self.permission;

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

/// Instruction builder for `UpdatePermission` via CPI with AccountView.
///
/// ### Accounts:
///   0. `[signer?]` authority - Either this or permissioned_account must be a signer
///   1. `[signer?]` permissioned_account - Either this or authority must be a signer
///   2. `[writable]` permission
#[derive(Clone, Debug)]
pub struct UpdatePermissionCpiBuilderAccountView<'a> {
    instruction: Box<UpdatePermissionCpiBuilderInstructionAccountView<'a>>,
}

impl<'a> UpdatePermissionCpiBuilderAccountView<'a> {
    pub fn new(program: &'a AccountView) -> Self {
        let instruction = Box::new(UpdatePermissionCpiBuilderInstructionAccountView {
            __program: program,
            authority: None,
            permissioned_account: None,
            permission: None,
            members: None,
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

    #[inline(always)]
    pub fn members(&mut self, members: Option<Vec<Member>>) -> &mut Self {
        self.instruction.members = members;
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
        let instruction = UpdatePermissionCpiAccountView {
            __program: self.instruction.__program,
            authority: self.instruction.authority.expect("authority is not set"),
            permissioned_account: self
                .instruction
                .permissioned_account
                .expect("permissioned_account is not set"),
            permission: self.instruction.permission.expect("permission is not set"),
        };
        instruction.invoke_with_remaining_accounts(
            self.instruction.members.as_deref(),
            &self.instruction.__remaining_accounts,
        )
    }
}

#[derive(Clone, Debug)]
struct UpdatePermissionCpiBuilderInstructionAccountView<'a> {
    __program: &'a AccountView,
    authority: Option<(&'a AccountView, bool)>,
    permissioned_account: Option<(&'a AccountView, bool)>,
    permission: Option<&'a AccountView>,
    members: Option<Vec<Member>>,
    /// Additional instruction accounts `(AccountView, is_writable, is_signer)`.
    __remaining_accounts: Vec<(&'a AccountView, bool, bool)>,
}
