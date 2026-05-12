use crate::compat::borsh::{self, BorshDeserialize, BorshSerialize};

use crate::access_control::structs::MembersArgs;
use crate::access_control::structs::Permission;
use crate::compat::{self, Compat, Modern};
use crate::consts::PERMISSION_PROGRAM_ID;
use solana_program::program::{invoke, invoke_signed};

pub const CREATE_PERMISSION_DISCRIMINATOR: u64 = 0;

/// Accounts.
#[derive(Debug)]
pub struct CreatePermission {
    pub permissioned_account: compat::Pubkey,

    pub permission: compat::Pubkey,

    pub payer: compat::Pubkey,

    pub system_program: compat::Pubkey,
}

impl CreatePermission {
    pub fn instruction(&self, args: CreatePermissionInstructionArgs) -> compat::Instruction {
        self.instruction_with_remaining_accounts(args, &[])
    }
    #[allow(clippy::arithmetic_side_effects)]
    #[allow(clippy::vec_init_then_push)]
    pub fn instruction_with_remaining_accounts(
        &self,
        args: CreatePermissionInstructionArgs,
        remaining_accounts: &[compat::AccountMeta],
    ) -> compat::Instruction {
        let mut accounts = Vec::with_capacity(4 + remaining_accounts.len());
        accounts.push(compat::AccountMeta::new_readonly(
            self.permissioned_account,
            true,
        ));
        accounts.push(compat::AccountMeta::new(self.permission, false));
        accounts.push(compat::AccountMeta::new(self.payer, true));
        accounts.push(compat::AccountMeta::new_readonly(
            self.system_program,
            false,
        ));
        accounts.extend_from_slice(remaining_accounts);
        let mut data = CreatePermissionInstructionData::new().try_to_vec().unwrap();
        let mut args = args.try_to_vec().unwrap();
        data.append(&mut args);

        compat::Instruction {
            program_id: PERMISSION_PROGRAM_ID,
            accounts,
            data,
        }
    }
}

#[derive(BorshSerialize, BorshDeserialize, Clone, Debug, Eq, PartialEq)]
#[cfg_attr(not(feature = "backward-compat"), borsh(crate = "crate::compat::borsh"))]
pub struct CreatePermissionInstructionData {
    discriminator: u64,
}

impl CreatePermissionInstructionData {
    pub fn new() -> Self {
        Self {
            discriminator: CREATE_PERMISSION_DISCRIMINATOR,
        }
    }

    pub(crate) fn try_to_vec(&self) -> Result<Vec<u8>, std::io::Error> {
        borsh::to_vec(self)
    }
}

impl Default for CreatePermissionInstructionData {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(BorshSerialize, BorshDeserialize, Clone, Debug, Eq, PartialEq)]
#[cfg_attr(not(feature = "backward-compat"), borsh(crate = "crate::compat::borsh"))]
pub struct CreatePermissionInstructionArgs {
    pub args: MembersArgs,
}

impl CreatePermissionInstructionArgs {
    pub(crate) fn try_to_vec(&self) -> Result<Vec<u8>, std::io::Error> {
        borsh::to_vec(self)
    }
}

/// compat::Instruction builder for `CreatePermission`.
///
/// ### Accounts:
///
///   0. `[signer]` permissioned_account
///   1. `[writable]` permission
///   2. `[writable, signer]` payer
///   3. `[]` system_program
#[derive(Clone, Debug, Default)]
pub struct CreatePermissionBuilder {
    permissioned_account: Option<compat::Pubkey>,
    permission: Option<compat::Pubkey>,
    payer: Option<compat::Pubkey>,
    system_program: Option<compat::Pubkey>,
    args: Option<MembersArgs>,
    __remaining_accounts: Vec<compat::AccountMeta>,
}

impl CreatePermissionBuilder {
    pub fn new() -> Self {
        Self::default()
    }
    #[inline(always)]
    pub fn permissioned_account(&mut self, permissioned_account: compat::Pubkey) -> &mut Self {
        self.permissioned_account = Some(permissioned_account);
        // Automatically derive and set the permission PDA
        let (permission_pda, _bump) = Permission::find_pda(&permissioned_account);
        self.permission = Some(permission_pda);
        self
    }
    #[inline(always)]
    pub fn permission(&mut self, permission: compat::Pubkey) -> &mut Self {
        self.permission = Some(permission);
        self
    }
    #[inline(always)]
    pub fn payer(&mut self, payer: compat::Pubkey) -> &mut Self {
        self.payer = Some(payer);
        self
    }
    #[inline(always)]
    pub fn system_program(&mut self, system_program: compat::Pubkey) -> &mut Self {
        self.system_program = Some(system_program);
        self
    }
    #[inline(always)]
    pub fn args(&mut self, args: MembersArgs) -> &mut Self {
        self.args = Some(args);
        self
    }
    /// Add an additional account to the instruction.
    #[inline(always)]
    pub fn add_remaining_account(&mut self, account: compat::AccountMeta) -> &mut Self {
        self.__remaining_accounts.push(account);
        self
    }
    /// Add additional accounts to the instruction.
    #[inline(always)]
    pub fn add_remaining_accounts(&mut self, accounts: &[compat::AccountMeta]) -> &mut Self {
        self.__remaining_accounts.extend_from_slice(accounts);
        self
    }
    #[allow(clippy::clone_on_copy)]
    pub fn instruction(&self) -> compat::Instruction {
        let accounts = CreatePermission {
            permissioned_account: self
                .permissioned_account
                .expect("permissioned_account is not set"),
            permission: self.permission.expect("permission is not set"),
            payer: self.payer.expect("payer is not set"),
            system_program: self.system_program.expect("system_program is not set"),
        };
        let args = CreatePermissionInstructionArgs {
            args: self.args.clone().expect("args is not set"),
        };

        accounts.instruction_with_remaining_accounts(args, &self.__remaining_accounts)
    }
}

/// `create_permission` CPI accounts.
pub struct CreatePermissionCpiAccounts<'a, 'b> {
    pub permissioned_account: &'b compat::AccountInfo<'a>,

    pub permission: &'b compat::AccountInfo<'a>,

    pub payer: &'b compat::AccountInfo<'a>,

    pub system_program: &'b compat::AccountInfo<'a>,
}

/// `create_permission` CPI instruction.
pub struct CreatePermissionCpi<'a, 'b> {
    /// The program to invoke.
    pub __program: &'b compat::AccountInfo<'a>,

    pub permissioned_account: &'b compat::AccountInfo<'a>,

    pub permission: &'b compat::AccountInfo<'a>,

    pub payer: &'b compat::AccountInfo<'a>,

    pub system_program: &'b compat::AccountInfo<'a>,
    /// The arguments for the instruction.
    pub __args: CreatePermissionInstructionArgs,
}

impl<'a, 'b> CreatePermissionCpi<'a, 'b> {
    pub fn new(
        program: &'b compat::AccountInfo<'a>,
        accounts: CreatePermissionCpiAccounts<'a, 'b>,
        args: CreatePermissionInstructionArgs,
    ) -> Self {
        Self {
            __program: program,
            permissioned_account: accounts.permissioned_account,
            permission: accounts.permission,
            payer: accounts.payer,
            system_program: accounts.system_program,
            __args: args,
        }
    }
    #[inline(always)]
    pub fn invoke(&self) -> compat::ProgramResult {
        self.invoke_signed_with_remaining_accounts(&[], &[])
    }
    #[inline(always)]
    pub fn invoke_with_remaining_accounts(
        &self,
        remaining_accounts: &[(&'b compat::AccountInfo<'a>, bool, bool)],
    ) -> compat::ProgramResult {
        self.invoke_signed_with_remaining_accounts(&[], remaining_accounts)
    }
    #[inline(always)]
    pub fn invoke_signed(&self, signers_seeds: &[&[&[u8]]]) -> compat::ProgramResult {
        self.invoke_signed_with_remaining_accounts(signers_seeds, &[])
    }
    #[allow(clippy::arithmetic_side_effects)]
    #[allow(clippy::clone_on_copy)]
    #[allow(clippy::vec_init_then_push)]
    pub fn invoke_signed_with_remaining_accounts(
        &self,
        signers_seeds: &[&[&[u8]]],
        remaining_accounts: &[(&'b compat::AccountInfo<'a>, bool, bool)],
    ) -> compat::ProgramResult {
        let mut accounts = Vec::with_capacity(4 + remaining_accounts.len());
        accounts.push(compat::AccountMeta::new_readonly(
            *self.permissioned_account.key,
            true,
        ));
        accounts.push(compat::AccountMeta::new(*self.permission.key, false));
        accounts.push(compat::AccountMeta::new(*self.payer.key, true));
        accounts.push(compat::AccountMeta::new_readonly(
            *self.system_program.key,
            false,
        ));
        remaining_accounts.iter().for_each(|remaining_account| {
            accounts.push(compat::AccountMeta {
                pubkey: *remaining_account.0.key,
                is_signer: remaining_account.2,
                is_writable: remaining_account.1,
            })
        });
        let mut data = CreatePermissionInstructionData::new().try_to_vec().unwrap();
        let mut args = self.__args.try_to_vec().unwrap();
        data.append(&mut args);

        let instruction = compat::Instruction {
            program_id: PERMISSION_PROGRAM_ID,
            accounts,
            data,
        };
        let mut account_infos = Vec::with_capacity(5 + remaining_accounts.len());
        account_infos.push(self.__program.clone());
        account_infos.push(self.permissioned_account.clone());
        account_infos.push(self.permission.clone());
        account_infos.push(self.payer.clone());
        account_infos.push(self.system_program.clone());
        remaining_accounts
            .iter()
            .for_each(|remaining_account| account_infos.push(remaining_account.0.clone()));

        if signers_seeds.is_empty() {
            invoke(&instruction.modern(), &account_infos.modern()).compat()
        } else {
            invoke_signed(
                &instruction.modern(),
                &account_infos.modern(),
                signers_seeds,
            )
            .compat()
        }
    }
}

/// compat::Instruction builder for `CreatePermission` via CPI.
///
/// ### Accounts:
///
///   0. `[signer]` permissioned_account
///   1. `[writable]` permission
///   2. `[writable, signer]` payer
///   3. `[]` system_program
#[derive(Clone, Debug)]
pub struct CreatePermissionCpiBuilder<'a, 'b> {
    instruction: Box<CreatePermissionCpiBuilderInstruction<'a, 'b>>,
}

impl<'a, 'b> CreatePermissionCpiBuilder<'a, 'b> {
    pub fn new(program: &'b compat::AccountInfo<'a>) -> Self {
        let instruction = Box::new(CreatePermissionCpiBuilderInstruction {
            __program: program,
            permissioned_account: None,
            permission: None,
            payer: None,
            system_program: None,
            args: None,
            __remaining_accounts: Vec::new(),
        });
        Self { instruction }
    }
    #[inline(always)]
    pub fn permissioned_account(
        &mut self,
        permissioned_account: &'b compat::AccountInfo<'a>,
    ) -> &mut Self {
        self.instruction.permissioned_account = Some(permissioned_account);
        self
    }
    #[inline(always)]
    pub fn permission(&mut self, permission: &'b compat::AccountInfo<'a>) -> &mut Self {
        self.instruction.permission = Some(permission);
        self
    }
    #[inline(always)]
    pub fn payer(&mut self, payer: &'b compat::AccountInfo<'a>) -> &mut Self {
        self.instruction.payer = Some(payer);
        self
    }
    #[inline(always)]
    pub fn system_program(&mut self, system_program: &'b compat::AccountInfo<'a>) -> &mut Self {
        self.instruction.system_program = Some(system_program);
        self
    }
    #[inline(always)]
    pub fn args(&mut self, args: MembersArgs) -> &mut Self {
        self.instruction.args = Some(args);
        self
    }
    /// Add an additional account to the instruction.
    #[inline(always)]
    pub fn add_remaining_account(
        &mut self,
        account: &'b compat::AccountInfo<'a>,
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
    /// Each account is represented by a tuple of the `compat::AccountInfo`, a `bool` indicating whether the account is writable or not,
    /// and a `bool` indicating whether the account is a signer or not.
    #[inline(always)]
    pub fn add_remaining_accounts(
        &mut self,
        accounts: &[(&'b compat::AccountInfo<'a>, bool, bool)],
    ) -> &mut Self {
        self.instruction
            .__remaining_accounts
            .extend_from_slice(accounts);
        self
    }
    #[inline(always)]
    pub fn invoke(&self) -> compat::ProgramResult {
        self.invoke_signed(&[])
    }
    #[allow(clippy::clone_on_copy)]
    #[allow(clippy::vec_init_then_push)]
    pub fn invoke_signed(&self, signers_seeds: &[&[&[u8]]]) -> compat::ProgramResult {
        let args = CreatePermissionInstructionArgs {
            args: self.instruction.args.clone().expect("args is not set"),
        };
        let instruction = CreatePermissionCpi {
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
            __args: args,
        };
        instruction.invoke_signed_with_remaining_accounts(
            signers_seeds,
            &self.instruction.__remaining_accounts,
        )
    }
}

#[derive(Clone, Debug)]
struct CreatePermissionCpiBuilderInstruction<'a, 'b> {
    __program: &'b compat::AccountInfo<'a>,
    permissioned_account: Option<&'b compat::AccountInfo<'a>>,
    permission: Option<&'b compat::AccountInfo<'a>>,
    payer: Option<&'b compat::AccountInfo<'a>>,
    system_program: Option<&'b compat::AccountInfo<'a>>,
    args: Option<MembersArgs>,
    /// Additional instruction accounts `(compat::AccountInfo, is_writable, is_signer)`.
    __remaining_accounts: Vec<(&'b compat::AccountInfo<'a>, bool, bool)>,
}
