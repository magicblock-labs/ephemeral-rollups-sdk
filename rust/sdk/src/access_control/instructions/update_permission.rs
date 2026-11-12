use borsh::{self, BorshDeserialize, BorshSerialize};

use crate::access_control::structs::MembersArgs;
use crate::access_control::structs::Permission;

use crate::consts::PERMISSION_PROGRAM_ID;
use solana_program::program::{invoke, invoke_signed};

pub const UPDATE_PERMISSION_DISCRIMINATOR: u64 = 1;

/// Accounts.
#[derive(Debug)]
pub struct UpdatePermission {
    pub authority: (solana_program::pubkey::Pubkey, bool),

    pub permissioned_account: (solana_program::pubkey::Pubkey, bool),

    pub permission: solana_program::pubkey::Pubkey,
}

impl UpdatePermission {
    pub fn instruction(
        &self,
        args: UpdatePermissionInstructionArgs,
    ) -> solana_program::instruction::Instruction {
        self.instruction_with_remaining_accounts(args, &[])
    }
    #[allow(clippy::arithmetic_side_effects)]
    #[allow(clippy::vec_init_then_push)]
    pub fn instruction_with_remaining_accounts(
        &self,
        args: UpdatePermissionInstructionArgs,
        remaining_accounts: &[solana_program::instruction::AccountMeta],
    ) -> solana_program::instruction::Instruction {
        let mut accounts = Vec::with_capacity(3 + remaining_accounts.len());
        accounts.push(solana_program::instruction::AccountMeta::new_readonly(
            self.authority.0,
            self.authority.1,
        ));
        accounts.push(solana_program::instruction::AccountMeta::new_readonly(
            self.permissioned_account.0,
            self.permissioned_account.1,
        ));
        accounts.push(solana_program::instruction::AccountMeta::new(
            self.permission,
            false,
        ));
        accounts.extend_from_slice(remaining_accounts);
        let mut data = UpdatePermissionInstructionData::new().try_to_vec().unwrap();
        let mut args = args.try_to_vec().unwrap();
        data.append(&mut args);

        solana_program::instruction::Instruction {
            program_id: PERMISSION_PROGRAM_ID,
            accounts,
            data,
        }
    }
}

#[derive(BorshSerialize, BorshDeserialize, Clone, Debug, Eq, PartialEq)]
#[borsh(crate = "borsh")]
pub struct UpdatePermissionInstructionData {
    discriminator: u64,
}

impl UpdatePermissionInstructionData {
    pub fn new() -> Self {
        Self {
            discriminator: UPDATE_PERMISSION_DISCRIMINATOR,
        }
    }

    pub(crate) fn try_to_vec(&self) -> Result<Vec<u8>, std::io::Error> {
        borsh::to_vec(self)
    }
}

impl Default for UpdatePermissionInstructionData {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(BorshSerialize, BorshDeserialize, Clone, Debug, Eq, PartialEq)]
#[borsh(crate = "borsh")]
pub struct UpdatePermissionInstructionArgs {
    pub args: MembersArgs,
}

impl UpdatePermissionInstructionArgs {
    pub(crate) fn try_to_vec(&self) -> Result<Vec<u8>, std::io::Error> {
        borsh::to_vec(self)
    }
}

/// solana_program::instruction::Instruction builder for `UpdatePermission`.
///
/// ### Accounts:
///
///   0. `[signer?]` authority - Either this or permissioned_account must be a signer
///   1. `[signer?]` permissioned_account - Either this or authority must be a signer
///   2. `[writable]` permission
#[derive(Clone, Debug, Default)]
pub struct UpdatePermissionBuilder {
    authority: Option<(solana_program::pubkey::Pubkey, bool)>,
    permissioned_account: Option<(solana_program::pubkey::Pubkey, bool)>,
    permission: Option<solana_program::pubkey::Pubkey>,
    args: Option<MembersArgs>,
    __remaining_accounts: Vec<solana_program::instruction::AccountMeta>,
}

impl UpdatePermissionBuilder {
    pub fn new() -> Self {
        Self::default()
    }
    #[inline(always)]
    pub fn authority(
        &mut self,
        authority: solana_program::pubkey::Pubkey,
        as_signer: bool,
    ) -> &mut Self {
        self.authority = Some((authority, as_signer));
        self
    }
    #[inline(always)]
    pub fn permissioned_account(
        &mut self,
        permissioned_account: solana_program::pubkey::Pubkey,
        as_signer: bool,
    ) -> &mut Self {
        self.permissioned_account = Some((permissioned_account, as_signer));
        // Automatically derive and set the permission PDA
        let (permission_pda, _bump) = Permission::find_pda(&permissioned_account);
        self.permission = Some(permission_pda);
        self
    }
    #[inline(always)]
    pub fn permission(&mut self, permission: solana_program::pubkey::Pubkey) -> &mut Self {
        self.permission = Some(permission);
        self
    }
    #[inline(always)]
    pub fn args(&mut self, args: MembersArgs) -> &mut Self {
        self.args = Some(args);
        self
    }
    /// Add an additional account to the instruction.
    #[inline(always)]
    pub fn add_remaining_account(
        &mut self,
        account: solana_program::instruction::AccountMeta,
    ) -> &mut Self {
        self.__remaining_accounts.push(account);
        self
    }
    /// Add additional accounts to the instruction.
    #[inline(always)]
    pub fn add_remaining_accounts(
        &mut self,
        accounts: &[solana_program::instruction::AccountMeta],
    ) -> &mut Self {
        self.__remaining_accounts.extend_from_slice(accounts);
        self
    }
    #[allow(clippy::clone_on_copy)]
    pub fn instruction(&self) -> solana_program::instruction::Instruction {
        let accounts = UpdatePermission {
            authority: self.authority.expect("authority is not set"),
            permissioned_account: self
                .permissioned_account
                .expect("permissioned_account is not set"),
            permission: self.permission.expect("permission is not set"),
        };
        let args = UpdatePermissionInstructionArgs {
            args: self.args.clone().expect("args is not set"),
        };

        accounts.instruction_with_remaining_accounts(args, &self.__remaining_accounts)
    }
}

/// `update_permission` CPI accounts.
pub struct UpdatePermissionCpiAccounts<'a, 'b> {
    pub authority: (&'b solana_program::account_info::AccountInfo<'a>, bool),

    pub permissioned_account: (&'b solana_program::account_info::AccountInfo<'a>, bool),

    pub permission: &'b solana_program::account_info::AccountInfo<'a>,
}

/// `update_permission` CPI instruction.
pub struct UpdatePermissionCpi<'a, 'b> {
    /// The program to invoke.
    pub __program: &'b solana_program::account_info::AccountInfo<'a>,

    pub authority: (&'b solana_program::account_info::AccountInfo<'a>, bool),

    pub permissioned_account: (&'b solana_program::account_info::AccountInfo<'a>, bool),

    pub permission: &'b solana_program::account_info::AccountInfo<'a>,
    /// The arguments for the instruction.
    pub __args: UpdatePermissionInstructionArgs,
}

impl<'a, 'b> UpdatePermissionCpi<'a, 'b> {
    pub fn new(
        program: &'b solana_program::account_info::AccountInfo<'a>,
        accounts: UpdatePermissionCpiAccounts<'a, 'b>,
        args: UpdatePermissionInstructionArgs,
    ) -> Self {
        Self {
            __program: program,
            authority: accounts.authority,
            permissioned_account: accounts.permissioned_account,
            permission: accounts.permission,
            __args: args,
        }
    }
    #[inline(always)]
    pub fn invoke(&self) -> solana_program::entrypoint_deprecated::ProgramResult {
        self.invoke_signed_with_remaining_accounts(&[], &[])
    }
    #[inline(always)]
    pub fn invoke_with_remaining_accounts(
        &self,
        remaining_accounts: &[(
            &'b solana_program::account_info::AccountInfo<'a>,
            bool,
            bool,
        )],
    ) -> solana_program::entrypoint_deprecated::ProgramResult {
        self.invoke_signed_with_remaining_accounts(&[], remaining_accounts)
    }
    #[inline(always)]
    pub fn invoke_signed(
        &self,
        signers_seeds: &[&[&[u8]]],
    ) -> solana_program::entrypoint_deprecated::ProgramResult {
        self.invoke_signed_with_remaining_accounts(signers_seeds, &[])
    }
    #[allow(clippy::arithmetic_side_effects)]
    #[allow(clippy::clone_on_copy)]
    #[allow(clippy::vec_init_then_push)]
    pub fn invoke_signed_with_remaining_accounts(
        &self,
        signers_seeds: &[&[&[u8]]],
        remaining_accounts: &[(
            &'b solana_program::account_info::AccountInfo<'a>,
            bool,
            bool,
        )],
    ) -> solana_program::entrypoint_deprecated::ProgramResult {
        let mut accounts = Vec::with_capacity(3 + remaining_accounts.len());
        accounts.push(solana_program::instruction::AccountMeta::new_readonly(
            *self.authority.0.key,
            self.authority.1,
        ));
        accounts.push(solana_program::instruction::AccountMeta::new_readonly(
            *self.permissioned_account.0.key,
            self.permissioned_account.1,
        ));
        accounts.push(solana_program::instruction::AccountMeta::new(
            *self.permission.key,
            false,
        ));
        remaining_accounts.iter().for_each(|remaining_account| {
            accounts.push(solana_program::instruction::AccountMeta {
                pubkey: *remaining_account.0.key,
                is_signer: remaining_account.2,
                is_writable: remaining_account.1,
            })
        });
        let mut data = UpdatePermissionInstructionData::new().try_to_vec().unwrap();
        let mut args = self.__args.try_to_vec().unwrap();
        data.append(&mut args);

        let instruction = solana_program::instruction::Instruction {
            program_id: PERMISSION_PROGRAM_ID,
            accounts,
            data,
        };
        let mut account_infos = Vec::with_capacity(4 + remaining_accounts.len());
        account_infos.push(self.__program.clone());
        account_infos.push(self.authority.0.clone());
        account_infos.push(self.permissioned_account.0.clone());
        account_infos.push(self.permission.clone());
        remaining_accounts
            .iter()
            .for_each(|remaining_account| account_infos.push(remaining_account.0.clone()));

        if signers_seeds.is_empty() {
            invoke(&instruction, &account_infos)
        } else {
            invoke_signed(&instruction, &account_infos, signers_seeds)
        }
    }
}

/// solana_program::instruction::Instruction builder for `UpdatePermission` via CPI.
///
/// ### Accounts:
///
///   0. `[signer?]` authority - Either this or permissioned_account must be a signer
///   1. `[signer?]` permissioned_account - Either this or authority must be a signer
///   2. `[writable]` permission
#[derive(Clone, Debug)]
pub struct UpdatePermissionCpiBuilder<'a, 'b> {
    instruction: Box<UpdatePermissionCpiBuilderInstruction<'a, 'b>>,
}

impl<'a, 'b> UpdatePermissionCpiBuilder<'a, 'b> {
    pub fn new(program: &'b solana_program::account_info::AccountInfo<'a>) -> Self {
        let instruction = Box::new(UpdatePermissionCpiBuilderInstruction {
            __program: program,
            authority: None,
            permissioned_account: None,
            permission: None,
            args: None,
            __remaining_accounts: Vec::new(),
        });
        Self { instruction }
    }
    #[inline(always)]
    pub fn authority(
        &mut self,
        authority: &'b solana_program::account_info::AccountInfo<'a>,
        as_signer: bool,
    ) -> &mut Self {
        self.instruction.authority = Some((authority, as_signer));
        self
    }
    #[inline(always)]
    pub fn permissioned_account(
        &mut self,
        permissioned_account: &'b solana_program::account_info::AccountInfo<'a>,
        as_signer: bool,
    ) -> &mut Self {
        self.instruction.permissioned_account = Some((permissioned_account, as_signer));
        self
    }
    #[inline(always)]
    pub fn permission(
        &mut self,
        permission: &'b solana_program::account_info::AccountInfo<'a>,
    ) -> &mut Self {
        self.instruction.permission = Some(permission);
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
        account: &'b solana_program::account_info::AccountInfo<'a>,
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
    /// Each account is represented by a tuple of the `solana_program::account_info::AccountInfo`, a `bool` indicating whether the account is writable or not,
    /// and a `bool` indicating whether the account is a signer or not.
    #[inline(always)]
    pub fn add_remaining_accounts(
        &mut self,
        accounts: &[(
            &'b solana_program::account_info::AccountInfo<'a>,
            bool,
            bool,
        )],
    ) -> &mut Self {
        self.instruction
            .__remaining_accounts
            .extend_from_slice(accounts);
        self
    }
    #[inline(always)]
    pub fn invoke(&self) -> solana_program::entrypoint_deprecated::ProgramResult {
        self.invoke_signed(&[])
    }
    #[allow(clippy::clone_on_copy)]
    #[allow(clippy::vec_init_then_push)]
    pub fn invoke_signed(
        &self,
        signers_seeds: &[&[&[u8]]],
    ) -> solana_program::entrypoint_deprecated::ProgramResult {
        let args = UpdatePermissionInstructionArgs {
            args: self.instruction.args.clone().expect("args is not set"),
        };
        let instruction = UpdatePermissionCpi {
            __program: self.instruction.__program,

            authority: self.instruction.authority.expect("authority is not set"),

            permissioned_account: self
                .instruction
                .permissioned_account
                .expect("permissioned_account is not set"),

            permission: self.instruction.permission.expect("permission is not set"),
            __args: args,
        };
        instruction.invoke_signed_with_remaining_accounts(
            signers_seeds,
            &self.instruction.__remaining_accounts,
        )
    }
}

#[derive(Clone, Debug)]
struct UpdatePermissionCpiBuilderInstruction<'a, 'b> {
    __program: &'b solana_program::account_info::AccountInfo<'a>,
    authority: Option<(&'b solana_program::account_info::AccountInfo<'a>, bool)>,
    permissioned_account: Option<(&'b solana_program::account_info::AccountInfo<'a>, bool)>,
    permission: Option<&'b solana_program::account_info::AccountInfo<'a>>,
    args: Option<MembersArgs>,
    /// Additional instruction accounts `(solana_program::account_info::AccountInfo, is_writable, is_signer)`.
    __remaining_accounts: Vec<(
        &'b solana_program::account_info::AccountInfo<'a>,
        bool,
        bool,
    )>,
}
