use borsh::{BorshDeserialize, BorshSerialize};

use crate::access_control::structs::MembersArgs;
use crate::access_control::structs::Permission;
use crate::consts::PERMISSION_PROGRAM_ID;
use crate::solana_compat::solana::{
    invoke, invoke_signed, AccountInfo, AccountMeta, Instruction, ProgramResult, Pubkey,
};

pub const CREATE_PERMISSION_DISCRIMINATOR: u64 = 0;

/// Accounts.
#[derive(Debug)]
pub struct CreatePermission {
    pub permissioned_account: Pubkey,

    pub permission: Pubkey,

    pub payer: Pubkey,

    pub system_program: Pubkey,
}

impl CreatePermission {
    pub fn instruction(&self, args: CreatePermissionInstructionArgs) -> Instruction {
        self.instruction_with_remaining_accounts(args, &[])
    }
    #[allow(clippy::arithmetic_side_effects)]
    #[allow(clippy::vec_init_then_push)]
    pub fn instruction_with_remaining_accounts(
        &self,
        args: CreatePermissionInstructionArgs,
        remaining_accounts: &[AccountMeta],
    ) -> Instruction {
        let mut accounts = Vec::with_capacity(4 + remaining_accounts.len());
        accounts.push(AccountMeta::new_readonly(self.permissioned_account, true));
        accounts.push(AccountMeta::new(self.permission, false));
        accounts.push(AccountMeta::new(self.payer, true));
        accounts.push(AccountMeta::new_readonly(self.system_program, false));
        accounts.extend_from_slice(remaining_accounts);
        let mut data = CreatePermissionInstructionData::new().try_to_vec().unwrap();
        let mut args = args.try_to_vec().unwrap();
        data.append(&mut args);

        Instruction {
            program_id: PERMISSION_PROGRAM_ID,
            accounts,
            data,
        }
    }
}

#[derive(BorshSerialize, BorshDeserialize, Clone, Debug, Eq, PartialEq)]
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
pub struct CreatePermissionInstructionArgs {
    pub args: MembersArgs,
}

impl CreatePermissionInstructionArgs {
    pub(crate) fn try_to_vec(&self) -> Result<Vec<u8>, std::io::Error> {
        borsh::to_vec(self)
    }
}

/// Instruction builder for `CreatePermission`.
///
/// ### Accounts:
///
///   0. `[signer]` permissioned_account
///   1. `[writable]` permission
///   2. `[writable, signer]` payer
///   3. `[]` system_program
#[derive(Clone, Debug, Default)]
pub struct CreatePermissionBuilder {
    permissioned_account: Option<Pubkey>,
    permission: Option<Pubkey>,
    payer: Option<Pubkey>,
    system_program: Option<Pubkey>,
    args: Option<MembersArgs>,
    __remaining_accounts: Vec<AccountMeta>,
}

impl CreatePermissionBuilder {
    pub fn new() -> Self {
        Self::default()
    }
    #[inline(always)]
    pub fn permissioned_account(&mut self, permissioned_account: Pubkey) -> &mut Self {
        self.permissioned_account = Some(permissioned_account);
        // Automatically derive and set the permission PDA
        let (permission_pda, _bump) = Permission::find_pda(&permissioned_account);
        self.permission = Some(permission_pda);
        self
    }
    #[inline(always)]
    pub fn permission(&mut self, permission: Pubkey) -> &mut Self {
        self.permission = Some(permission);
        self
    }
    #[inline(always)]
    pub fn payer(&mut self, payer: Pubkey) -> &mut Self {
        self.payer = Some(payer);
        self
    }
    #[inline(always)]
    pub fn system_program(&mut self, system_program: Pubkey) -> &mut Self {
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
    pub fn add_remaining_account(&mut self, account: AccountMeta) -> &mut Self {
        self.__remaining_accounts.push(account);
        self
    }
    /// Add additional accounts to the instruction.
    #[inline(always)]
    pub fn add_remaining_accounts(&mut self, accounts: &[AccountMeta]) -> &mut Self {
        self.__remaining_accounts.extend_from_slice(accounts);
        self
    }
    #[allow(clippy::clone_on_copy)]
    pub fn instruction(&self) -> Instruction {
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
    pub permissioned_account: &'b AccountInfo<'a>,

    pub permission: &'b AccountInfo<'a>,

    pub payer: &'b AccountInfo<'a>,

    pub system_program: &'b AccountInfo<'a>,
}

/// `create_permission` CPI instruction.
pub struct CreatePermissionCpi<'a, 'b> {
    /// The program to invoke.
    pub __program: &'b AccountInfo<'a>,

    pub permissioned_account: &'b AccountInfo<'a>,

    pub permission: &'b AccountInfo<'a>,

    pub payer: &'b AccountInfo<'a>,

    pub system_program: &'b AccountInfo<'a>,
    /// The arguments for the instruction.
    pub __args: CreatePermissionInstructionArgs,
}

impl<'a, 'b> CreatePermissionCpi<'a, 'b> {
    pub fn new(
        program: &'b AccountInfo<'a>,
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
    pub fn invoke(&self) -> ProgramResult {
        self.invoke_signed_with_remaining_accounts(&[], &[])
    }
    #[inline(always)]
    pub fn invoke_with_remaining_accounts(
        &self,
        remaining_accounts: &[(&'b AccountInfo<'a>, bool, bool)],
    ) -> ProgramResult {
        self.invoke_signed_with_remaining_accounts(&[], remaining_accounts)
    }
    #[inline(always)]
    pub fn invoke_signed(&self, signers_seeds: &[&[&[u8]]]) -> ProgramResult {
        self.invoke_signed_with_remaining_accounts(signers_seeds, &[])
    }
    #[allow(clippy::arithmetic_side_effects)]
    #[allow(clippy::clone_on_copy)]
    #[allow(clippy::vec_init_then_push)]
    pub fn invoke_signed_with_remaining_accounts(
        &self,
        signers_seeds: &[&[&[u8]]],
        remaining_accounts: &[(&'b AccountInfo<'a>, bool, bool)],
    ) -> ProgramResult {
        let mut accounts = Vec::with_capacity(4 + remaining_accounts.len());
        accounts.push(AccountMeta::new_readonly(
            *self.permissioned_account.key,
            true,
        ));
        accounts.push(AccountMeta::new(*self.permission.key, false));
        accounts.push(AccountMeta::new(*self.payer.key, true));
        accounts.push(AccountMeta::new_readonly(*self.system_program.key, false));
        remaining_accounts.iter().for_each(|remaining_account| {
            accounts.push(AccountMeta {
                pubkey: *remaining_account.0.key,
                is_signer: remaining_account.2,
                is_writable: remaining_account.1,
            })
        });
        let mut data = CreatePermissionInstructionData::new().try_to_vec().unwrap();
        let mut args = self.__args.try_to_vec().unwrap();
        data.append(&mut args);

        let instruction = Instruction {
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
            invoke(&instruction, &account_infos)
        } else {
            invoke_signed(&instruction, &account_infos, signers_seeds)
        }
    }
}

/// Instruction builder for `CreatePermission` via CPI.
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
    pub fn new(program: &'b AccountInfo<'a>) -> Self {
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
    pub fn permissioned_account(&mut self, permissioned_account: &'b AccountInfo<'a>) -> &mut Self {
        self.instruction.permissioned_account = Some(permissioned_account);
        self
    }
    #[inline(always)]
    pub fn permission(&mut self, permission: &'b AccountInfo<'a>) -> &mut Self {
        self.instruction.permission = Some(permission);
        self
    }
    #[inline(always)]
    pub fn payer(&mut self, payer: &'b AccountInfo<'a>) -> &mut Self {
        self.instruction.payer = Some(payer);
        self
    }
    #[inline(always)]
    pub fn system_program(&mut self, system_program: &'b AccountInfo<'a>) -> &mut Self {
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
        account: &'b AccountInfo<'a>,
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
    /// Each account is represented by a tuple of the `AccountInfo`, a `bool` indicating whether the account is writable or not,
    /// and a `bool` indicating whether the account is a signer or not.
    #[inline(always)]
    pub fn add_remaining_accounts(
        &mut self,
        accounts: &[(&'b AccountInfo<'a>, bool, bool)],
    ) -> &mut Self {
        self.instruction
            .__remaining_accounts
            .extend_from_slice(accounts);
        self
    }
    #[inline(always)]
    pub fn invoke(&self) -> ProgramResult {
        self.invoke_signed(&[])
    }
    #[allow(clippy::clone_on_copy)]
    #[allow(clippy::vec_init_then_push)]
    pub fn invoke_signed(&self, signers_seeds: &[&[&[u8]]]) -> ProgramResult {
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
    __program: &'b AccountInfo<'a>,
    permissioned_account: Option<&'b AccountInfo<'a>>,
    permission: Option<&'b AccountInfo<'a>>,
    payer: Option<&'b AccountInfo<'a>>,
    system_program: Option<&'b AccountInfo<'a>>,
    args: Option<MembersArgs>,
    /// Additional instruction accounts `(AccountInfo, is_writable, is_signer)`.
    __remaining_accounts: Vec<(&'b AccountInfo<'a>, bool, bool)>,
}
