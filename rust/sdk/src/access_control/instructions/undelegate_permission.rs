use crate::compat::borsh::{self, BorshDeserialize, BorshSerialize};

use crate::access_control::structs::UndelegateArgs;
use crate::compat::{self, Compat, Modern};
use crate::consts::PERMISSION_PROGRAM_ID;
use solana_program::program::{invoke, invoke_signed};

pub const UNDELEGATE_PERMISSION_DISCRIMINATOR: u64 = 12048014319693667524;

/// Accounts.
#[derive(Debug)]
pub struct UndelegatePermission {
    pub delegated_permission: compat::Pubkey,

    pub delegation_buffer: compat::Pubkey,

    pub validator: compat::Pubkey,

    pub system_program: compat::Pubkey,
}

impl UndelegatePermission {
    pub fn instruction(&self, args: UndelegatePermissionInstructionArgs) -> compat::Instruction {
        self.instruction_with_remaining_accounts(args, &[])
    }
    #[allow(clippy::arithmetic_side_effects)]
    #[allow(clippy::vec_init_then_push)]
    pub fn instruction_with_remaining_accounts(
        &self,
        args: UndelegatePermissionInstructionArgs,
        remaining_accounts: &[compat::AccountMeta],
    ) -> compat::Instruction {
        let mut accounts = Vec::with_capacity(4 + remaining_accounts.len());
        accounts.push(compat::AccountMeta::new(self.delegated_permission, false));
        accounts.push(compat::AccountMeta::new(self.delegation_buffer, false));
        accounts.push(compat::AccountMeta::new_readonly(self.validator, true));
        accounts.push(compat::AccountMeta::new_readonly(
            self.system_program,
            false,
        ));
        accounts.extend_from_slice(remaining_accounts);
        let mut data = UndelegatePermissionInstructionData::new()
            .try_to_vec()
            .unwrap();
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
pub struct UndelegatePermissionInstructionData {
    discriminator: u64,
}

impl UndelegatePermissionInstructionData {
    pub fn new() -> Self {
        Self {
            discriminator: UNDELEGATE_PERMISSION_DISCRIMINATOR,
        }
    }

    pub(crate) fn try_to_vec(&self) -> Result<Vec<u8>, std::io::Error> {
        borsh::to_vec(self)
    }
}

impl Default for UndelegatePermissionInstructionData {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(BorshSerialize, BorshDeserialize, Clone, Debug, Eq, PartialEq)]
#[cfg_attr(not(feature = "backward-compat"), borsh(crate = "crate::compat::borsh"))]
pub struct UndelegatePermissionInstructionArgs {
    pub args: UndelegateArgs,
}

impl UndelegatePermissionInstructionArgs {
    pub(crate) fn try_to_vec(&self) -> Result<Vec<u8>, std::io::Error> {
        borsh::to_vec(self)
    }
}

/// compat::Instruction builder for `UndelegatePermission`.
///
/// ### Accounts:
///
///   0. `[writable]` delegated_permission
///   1. `[writable]` delegation_buffer
///   2. `[signer]` validator
///   3. `[]` system_program
#[derive(Clone, Debug, Default)]
pub struct UndelegatePermissionBuilder {
    delegated_permission: Option<compat::Pubkey>,
    delegation_buffer: Option<compat::Pubkey>,
    validator: Option<compat::Pubkey>,
    system_program: Option<compat::Pubkey>,
    args: Option<UndelegateArgs>,
    __remaining_accounts: Vec<compat::AccountMeta>,
}

impl UndelegatePermissionBuilder {
    pub fn new() -> Self {
        Self::default()
    }
    #[inline(always)]
    pub fn delegated_permission(&mut self, delegated_permission: compat::Pubkey) -> &mut Self {
        self.delegated_permission = Some(delegated_permission);
        self
    }
    #[inline(always)]
    pub fn delegation_buffer(&mut self, delegation_buffer: compat::Pubkey) -> &mut Self {
        self.delegation_buffer = Some(delegation_buffer);
        self
    }
    #[inline(always)]
    pub fn validator(&mut self, validator: compat::Pubkey) -> &mut Self {
        self.validator = Some(validator);
        self
    }
    #[inline(always)]
    pub fn system_program(&mut self, system_program: compat::Pubkey) -> &mut Self {
        self.system_program = Some(system_program);
        self
    }
    #[inline(always)]
    pub fn args(&mut self, args: UndelegateArgs) -> &mut Self {
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
        let accounts = UndelegatePermission {
            delegated_permission: self
                .delegated_permission
                .expect("delegated_permission is not set"),
            delegation_buffer: self
                .delegation_buffer
                .expect("delegation_buffer is not set"),
            validator: self.validator.expect("validator is not set"),
            system_program: self.system_program.expect("system_program is not set"),
        };
        let args = UndelegatePermissionInstructionArgs {
            args: self.args.clone().expect("args is not set"),
        };

        accounts.instruction_with_remaining_accounts(args, &self.__remaining_accounts)
    }
}

/// `undelegate_permission` CPI accounts.
pub struct UndelegatePermissionCpiAccounts<'a, 'b> {
    pub delegated_permission: &'b compat::AccountInfo<'a>,

    pub delegation_buffer: &'b compat::AccountInfo<'a>,

    pub validator: &'b compat::AccountInfo<'a>,

    pub system_program: &'b compat::AccountInfo<'a>,
}

/// `undelegate_permission` CPI instruction.
pub struct UndelegatePermissionCpi<'a, 'b> {
    /// The program to invoke.
    pub __program: &'b compat::AccountInfo<'a>,

    pub delegated_permission: &'b compat::AccountInfo<'a>,

    pub delegation_buffer: &'b compat::AccountInfo<'a>,

    pub validator: &'b compat::AccountInfo<'a>,

    pub system_program: &'b compat::AccountInfo<'a>,
    /// The arguments for the instruction.
    pub __args: UndelegatePermissionInstructionArgs,
}

impl<'a, 'b> UndelegatePermissionCpi<'a, 'b> {
    pub fn new(
        program: &'b compat::AccountInfo<'a>,
        accounts: UndelegatePermissionCpiAccounts<'a, 'b>,
        args: UndelegatePermissionInstructionArgs,
    ) -> Self {
        Self {
            __program: program,
            delegated_permission: accounts.delegated_permission,
            delegation_buffer: accounts.delegation_buffer,
            validator: accounts.validator,
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
        accounts.push(compat::AccountMeta::new(
            *self.delegated_permission.key,
            false,
        ));
        accounts.push(compat::AccountMeta::new(*self.delegation_buffer.key, false));
        accounts.push(compat::AccountMeta::new_readonly(*self.validator.key, true));
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
        let mut data = UndelegatePermissionInstructionData::new()
            .try_to_vec()
            .unwrap();
        let mut args = self.__args.try_to_vec().unwrap();
        data.append(&mut args);

        let instruction = compat::Instruction {
            program_id: PERMISSION_PROGRAM_ID,
            accounts,
            data,
        };
        let mut account_infos = Vec::with_capacity(5 + remaining_accounts.len());
        account_infos.push(self.__program.clone());
        account_infos.push(self.delegated_permission.clone());
        account_infos.push(self.delegation_buffer.clone());
        account_infos.push(self.validator.clone());
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

/// compat::Instruction builder for `UndelegatePermission` via CPI.
///
/// ### Accounts:
///
///   0. `[writable]` delegated_permission
///   1. `[writable]` delegation_buffer
///   2. `[signer]` validator
///   3. `[]` system_program
#[derive(Clone, Debug)]
pub struct UndelegatePermissionCpiBuilder<'a, 'b> {
    instruction: Box<UndelegatePermissionCpiBuilderInstruction<'a, 'b>>,
}

impl<'a, 'b> UndelegatePermissionCpiBuilder<'a, 'b> {
    pub fn new(program: &'b compat::AccountInfo<'a>) -> Self {
        let instruction = Box::new(UndelegatePermissionCpiBuilderInstruction {
            __program: program,
            delegated_permission: None,
            delegation_buffer: None,
            validator: None,
            system_program: None,
            args: None,
            __remaining_accounts: Vec::new(),
        });
        Self { instruction }
    }
    #[inline(always)]
    pub fn delegated_permission(
        &mut self,
        delegated_permission: &'b compat::AccountInfo<'a>,
    ) -> &mut Self {
        self.instruction.delegated_permission = Some(delegated_permission);
        self
    }
    #[inline(always)]
    pub fn delegation_buffer(
        &mut self,
        delegation_buffer: &'b compat::AccountInfo<'a>,
    ) -> &mut Self {
        self.instruction.delegation_buffer = Some(delegation_buffer);
        self
    }
    #[inline(always)]
    pub fn validator(&mut self, validator: &'b compat::AccountInfo<'a>) -> &mut Self {
        self.instruction.validator = Some(validator);
        self
    }
    #[inline(always)]
    pub fn system_program(&mut self, system_program: &'b compat::AccountInfo<'a>) -> &mut Self {
        self.instruction.system_program = Some(system_program);
        self
    }
    #[inline(always)]
    pub fn args(&mut self, args: UndelegateArgs) -> &mut Self {
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
        let args = UndelegatePermissionInstructionArgs {
            args: self.instruction.args.clone().expect("args is not set"),
        };
        let instruction = UndelegatePermissionCpi {
            __program: self.instruction.__program,

            delegated_permission: self
                .instruction
                .delegated_permission
                .expect("delegated_permission is not set"),

            delegation_buffer: self
                .instruction
                .delegation_buffer
                .expect("delegation_buffer is not set"),

            validator: self.instruction.validator.expect("validator is not set"),

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
struct UndelegatePermissionCpiBuilderInstruction<'a, 'b> {
    __program: &'b compat::AccountInfo<'a>,
    delegated_permission: Option<&'b compat::AccountInfo<'a>>,
    delegation_buffer: Option<&'b compat::AccountInfo<'a>>,
    validator: Option<&'b compat::AccountInfo<'a>>,
    system_program: Option<&'b compat::AccountInfo<'a>>,
    args: Option<UndelegateArgs>,
    /// Additional instruction accounts `(compat::AccountInfo, is_writable, is_signer)`.
    __remaining_accounts: Vec<(&'b compat::AccountInfo<'a>, bool, bool)>,
}
