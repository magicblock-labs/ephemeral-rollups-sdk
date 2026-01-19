/// Create permission instruction builder
use crate::access_control::pinocchio::instructions::CREATE_PERMISSION_DISCRIMINATOR;
use crate::access_control::pinocchio::pda::permission_program_id;
use crate::access_control::pinocchio::structs::{Member, MembersArgs};
use pinocchio::{instruction::InstructionAccount, error::ProgramError, Address, AccountView, cpi::invoke_with_slice};
use core::mem::MaybeUninit;

const MAX_MEMBERS_SIZE: usize = 4 + (33 * 512);
const MAX_INSTRUCTION_DATA: usize = 8 + MAX_MEMBERS_SIZE;

/// Instruction builder for `CreatePermission`.
///
/// ### Accounts:
///   0. `[signer]` permissioned_account
///   1. `[writable]` permission (PDA)
///   2. `[writable, signer]` payer
///   3. `[]` system_program
#[derive(Clone, Debug, Default)]
pub struct CreatePermissionBuilder {
    permissioned_account: Option<Address>,
    permission: Option<Address>,
    payer: Option<Address>,
    system_program: Option<Address>,
    members: Option<Vec<Member>>,
}

impl CreatePermissionBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    #[inline(always)]
    pub fn permissioned_account(&mut self, permissioned_account: Address) -> &mut Self {
        self.permissioned_account = Some(permissioned_account);
        self
    }

    #[inline(always)]
    pub fn permission(&mut self, permission: Address) -> &mut Self {
        self.permission = Some(permission);
        self
    }

    #[inline(always)]
    pub fn payer(&mut self, payer: Address) -> &mut Self {
        self.payer = Some(payer);
        self
    }

    #[inline(always)]
    pub fn system_program(&mut self, system_program: Address) -> &mut Self {
        self.system_program = Some(system_program);
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
        [MaybeUninit<InstructionAccount>; 4],
        usize,
        Address,
        &'a [u8],
    ), ProgramError> {
        let permissioned_account = self
            .permissioned_account
            .as_ref()
            .ok_or(ProgramError::InvalidArgument)?;
        let permission = self.permission.as_ref().ok_or(ProgramError::InvalidArgument)?;
        let payer = self.payer.as_ref().ok_or(ProgramError::InvalidArgument)?;
        let system_program = self
            .system_program
            .as_ref()
            .ok_or(ProgramError::InvalidArgument)?;

        create_permission_instruction_impl(
            permissioned_account,
            permission,
            payer,
            system_program,
            self.members.as_deref(),
            data_buf,
        )
    }
}

/// Build a create_permission instruction
///
/// # Accounts
/// 0. `[signer]` permissioned_account
/// 1. `[writable]` permission (PDA)
/// 2. `[writable, signer]` payer
/// 3. `[]` system_program
fn create_permission_instruction_impl<'a>(
    permissioned_account: &Address,
    permission: &Address,
    payer: &Address,
    system_program: &Address,
    members: Option<&[Member]>,
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
            address: permissioned_account,
            is_writable: false,
            is_signer: true,
        });
        metas[1].write(InstructionAccount {
            address: permission,
            is_writable: true,
            is_signer: false,
        });
        metas[2].write(InstructionAccount {
            address: payer,
            is_writable: true,
            is_signer: true,
        });
        metas[3].write(InstructionAccount {
            address: system_program,
            is_writable: false,
            is_signer: false,
        });
    }

    // Serialize discriminator
    let discriminator_bytes = CREATE_PERMISSION_DISCRIMINATOR.to_le_bytes();
    data_buf[0..8].copy_from_slice(&discriminator_bytes);

    // Serialize members
    let members_len = serialize_members(&mut data_buf[8..], members)?;
    let total_len = 8 + members_len;

    Ok((metas, 4, permission_program_id(), &data_buf[..total_len]))
}

/// Serialize members into a buffer
/// Format: 4 bytes count + (1 byte flags + 32 bytes address) per member
fn serialize_members(buf: &mut [u8], members: Option<&[Member]>) -> Result<usize, ProgramError> {
    if buf.len() < 4 {
        return Err(ProgramError::InvalidArgument);
    }

    let member_count = members.map(|m| m.len()).unwrap_or(0);

    // Check size
    if 4 + member_count * 33 > buf.len() {
        return Err(ProgramError::InvalidArgument);
    }

    // Serialize count
    let count_bytes = (member_count as u32).to_le_bytes();
    buf[0..4].copy_from_slice(&count_bytes);

    let mut offset = 4;

    // Serialize members
    if let Some(members) = members {
        for member in members {
            // Flags (1 byte)
            buf[offset] = member.flags;
            offset += 1;

            // Address (32 bytes)
            buf[offset..offset + 32].copy_from_slice(member.pubkey.as_ref());
            offset += 32;
        }
    }

    Ok(offset)
}

/// `create_permission` CPI accounts using AccountView.
pub struct CreatePermissionCpiAccountsAccountView<'a> {
    pub permissioned_account: &'a AccountView,
    pub permission: &'a AccountView,
    pub payer: &'a AccountView,
    pub system_program: &'a AccountView,
}

/// `create_permission` CPI instruction using AccountView.
pub struct CreatePermissionCpiAccountView<'a> {
    /// The program to invoke.
    pub __program: &'a AccountView,
    pub permissioned_account: &'a AccountView,
    pub permission: &'a AccountView,
    pub payer: &'a AccountView,
    pub system_program: &'a AccountView,
}

impl<'a> CreatePermissionCpiAccountView<'a> {
    pub fn new(
        program: &'a AccountView,
        accounts: CreatePermissionCpiAccountsAccountView<'a>,
    ) -> Self {
        Self {
            __program: program,
            permissioned_account: accounts.permissioned_account,
            permission: accounts.permission,
            payer: accounts.payer,
            system_program: accounts.system_program,
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
        let (metas, num_accounts, program_id, data) = create_permission_instruction_impl(
            &self.permissioned_account.address,
            &self.permission.address,
            &self.payer.address,
            &self.system_program.address,
            members,
            &mut data_buf,
        )?;

        let mut all_accounts: [&AccountView; 8] = [self.__program; 8];
        all_accounts[0] = self.permissioned_account;
        all_accounts[1] = self.permission;
        all_accounts[2] = self.payer;
        all_accounts[3] = self.system_program;

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

/// Instruction builder for `CreatePermission` via CPI with AccountView.
///
/// ### Accounts:
///   0. `[signer]` permissioned_account
///   1. `[writable]` permission (PDA)
///   2. `[writable, signer]` payer
///   3. `[]` system_program
#[derive(Clone, Debug)]
pub struct CreatePermissionCpiBuilderAccountView<'a> {
    instruction: Box<CreatePermissionCpiBuilderInstructionAccountView<'a>>,
}

impl<'a> CreatePermissionCpiBuilderAccountView<'a> {
    pub fn new(program: &'a AccountView) -> Self {
        let instruction = Box::new(CreatePermissionCpiBuilderInstructionAccountView {
            __program: program,
            permissioned_account: None,
            permission: None,
            payer: None,
            system_program: None,
            members: None,
            __remaining_accounts: Vec::new(),
        });
        Self { instruction }
    }

    #[inline(always)]
    pub fn permissioned_account(&mut self, permissioned_account: &'a AccountView) -> &mut Self {
        self.instruction.permissioned_account = Some(permissioned_account);
        self
    }

    #[inline(always)]
    pub fn permission(&mut self, permission: &'a AccountView) -> &mut Self {
        self.instruction.permission = Some(permission);
        self
    }

    #[inline(always)]
    pub fn payer(&mut self, payer: &'a AccountView) -> &mut Self {
        self.instruction.payer = Some(payer);
        self
    }

    #[inline(always)]
    pub fn system_program(&mut self, system_program: &'a AccountView) -> &mut Self {
        self.instruction.system_program = Some(system_program);
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
        let instruction = CreatePermissionCpiAccountView {
            __program: self.instruction.__program,
            permissioned_account: self
                .instruction
                .permissioned_account
                .expect("permissioned_account is not set"),
            permission: self.instruction.permission.expect("permission is not set"),
            payer: self.instruction.payer.expect("payer is not set"),
            system_program: self
                .instruction
                .system_program
                .expect("system_program is not set"),
        };
        instruction.invoke_with_remaining_accounts(
            self.instruction.members.as_deref(),
            &self.instruction.__remaining_accounts,
        )
    }
}

#[derive(Clone, Debug)]
struct CreatePermissionCpiBuilderInstructionAccountView<'a> {
    __program: &'a AccountView,
    permissioned_account: Option<&'a AccountView>,
    permission: Option<&'a AccountView>,
    payer: Option<&'a AccountView>,
    system_program: Option<&'a AccountView>,
    members: Option<Vec<Member>>,
    /// Additional instruction accounts `(AccountView, is_writable, is_signer)`.
    __remaining_accounts: Vec<(&'a AccountView, bool, bool)>,
}
