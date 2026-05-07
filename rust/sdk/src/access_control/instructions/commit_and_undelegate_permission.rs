use crate::compat::borsh::{self, BorshDeserialize, BorshSerialize};

use crate::access_control::structs::Permission;
use crate::compat::{self, Compat, Modern};
use crate::consts::PERMISSION_PROGRAM_ID;
use crate::consts::{MAGIC_CONTEXT_ID, MAGIC_PROGRAM_ID};
use solana_program::program::{invoke, invoke_signed};

pub const COMMIT_AND_UNDELEGATE_PERMISSION_DISCRIMINATOR: u64 = 5;

/// Accounts.
#[derive(Debug)]
pub struct CommitAndUndelegatePermission {
    pub authority: (compat::Pubkey, bool),

    pub permissioned_account: (compat::Pubkey, bool),

    pub permission: compat::Pubkey,

    pub magic_program: compat::Pubkey,

    pub magic_context: compat::Pubkey,
}

impl CommitAndUndelegatePermission {
    pub fn instruction(&self) -> compat::Instruction {
        self.instruction_with_remaining_accounts(&[])
    }
    #[allow(clippy::arithmetic_side_effects)]
    #[allow(clippy::vec_init_then_push)]
    pub fn instruction_with_remaining_accounts(
        &self,
        remaining_accounts: &[compat::AccountMeta],
    ) -> compat::Instruction {
        let mut accounts = Vec::with_capacity(5 + remaining_accounts.len());
        accounts.push(compat::AccountMeta::new_readonly(
            self.authority.0,
            self.authority.1,
        ));
        accounts.push(compat::AccountMeta::new(
            self.permissioned_account.0,
            self.permissioned_account.1,
        ));
        accounts.push(compat::AccountMeta::new(self.permission, false));
        accounts.push(compat::AccountMeta::new_readonly(self.magic_program, false));
        accounts.push(compat::AccountMeta::new(self.magic_context, false));
        accounts.extend_from_slice(remaining_accounts);
        let data = CommitAndUndelegatePermissionInstructionData::new()
            .try_to_vec()
            .expect("failed to serialize CommitAndUndelegatePermissionInstructionData");

        compat::Instruction {
            program_id: PERMISSION_PROGRAM_ID,
            accounts,
            data,
        }
    }
}

#[derive(BorshSerialize, BorshDeserialize, Clone, Debug, Eq, PartialEq)]
pub struct CommitAndUndelegatePermissionInstructionData {
    discriminator: u64,
}

impl CommitAndUndelegatePermissionInstructionData {
    pub fn new() -> Self {
        Self {
            discriminator: COMMIT_AND_UNDELEGATE_PERMISSION_DISCRIMINATOR,
        }
    }

    pub(crate) fn try_to_vec(&self) -> Result<Vec<u8>, std::io::Error> {
        borsh::to_vec(self)
    }
}

impl Default for CommitAndUndelegatePermissionInstructionData {
    fn default() -> Self {
        Self::new()
    }
}

/// compat::Instruction builder for `CommitAndUndelegatePermission`.
///
/// ### Accounts (magic_program and magic_context are auto-set):
///
///   0. `[signer?]` authority - Either this or permissioned_account must be a signer
///   1. `[writable, signer?]` permissioned_account - Either this or authority must be a signer
///   2. `[writable]` permission
///   3. `[]` magic_program (auto-set to MAGIC_PROGRAM_ID)
///   4. `[writable]` magic_context (auto-set to MAGIC_CONTEXT_ID)
#[derive(Clone, Debug, Default)]
pub struct CommitAndUndelegatePermissionBuilder {
    authority: Option<(compat::Pubkey, bool)>,
    permissioned_account: Option<(compat::Pubkey, bool)>,
    permission: Option<compat::Pubkey>,
    __remaining_accounts: Vec<compat::AccountMeta>,
}

impl CommitAndUndelegatePermissionBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    #[inline(always)]
    pub fn authority(&mut self, authority: compat::Pubkey, as_signer: bool) -> &mut Self {
        self.authority = Some((authority, as_signer));
        self
    }
    #[inline(always)]
    pub fn permissioned_account(
        &mut self,
        permissioned_account: compat::Pubkey,
        as_signer: bool,
    ) -> &mut Self {
        self.permissioned_account = Some((permissioned_account, as_signer));
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
        let accounts = CommitAndUndelegatePermission {
            authority: self.authority.expect("authority is not set"),
            permissioned_account: self
                .permissioned_account
                .expect("permissioned_account is not set"),
            permission: self.permission.expect("permission is not set"),
            magic_program: MAGIC_PROGRAM_ID,
            magic_context: MAGIC_CONTEXT_ID,
        };

        accounts.instruction_with_remaining_accounts(&self.__remaining_accounts)
    }
}

/// `commit_and_undelegate_permission` CPI accounts.
pub struct CommitAndUndelegatePermissionCpiAccounts<'a, 'b> {
    pub authority: (&'b compat::AccountInfo<'a>, bool),

    pub permissioned_account: (&'b compat::AccountInfo<'a>, bool),

    pub permission: &'b compat::AccountInfo<'a>,

    pub magic_program: &'b compat::AccountInfo<'a>,

    pub magic_context: &'b compat::AccountInfo<'a>,
}

/// `commit_and_undelegate_permission` CPI instruction.
pub struct CommitAndUndelegatePermissionCpi<'a, 'b> {
    /// The program to invoke.
    pub __program: &'b compat::AccountInfo<'a>,

    pub authority: (&'b compat::AccountInfo<'a>, bool),

    pub permissioned_account: (&'b compat::AccountInfo<'a>, bool),

    pub permission: &'b compat::AccountInfo<'a>,

    pub magic_program: &'b compat::AccountInfo<'a>,

    pub magic_context: &'b compat::AccountInfo<'a>,
}

impl<'a, 'b> CommitAndUndelegatePermissionCpi<'a, 'b> {
    pub fn new(
        program: &'b compat::AccountInfo<'a>,
        accounts: CommitAndUndelegatePermissionCpiAccounts<'a, 'b>,
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
        let mut accounts = Vec::with_capacity(5 + remaining_accounts.len());
        accounts.push(compat::AccountMeta::new_readonly(
            *self.authority.0.key,
            self.authority.1,
        ));
        accounts.push(compat::AccountMeta::new(
            *self.permissioned_account.0.key,
            self.permissioned_account.1,
        ));
        accounts.push(compat::AccountMeta::new(*self.permission.key, false));
        accounts.push(compat::AccountMeta::new_readonly(
            *self.magic_program.key,
            false,
        ));
        accounts.push(compat::AccountMeta::new(*self.magic_context.key, false));
        remaining_accounts.iter().for_each(|remaining_account| {
            accounts.push(compat::AccountMeta {
                pubkey: *remaining_account.0.key,
                is_signer: remaining_account.2,
                is_writable: remaining_account.1,
            })
        });
        let data = CommitAndUndelegatePermissionInstructionData::new()
            .try_to_vec()
            .unwrap();

        let instruction = compat::Instruction {
            program_id: PERMISSION_PROGRAM_ID,
            accounts,
            data,
        };
        let mut account_infos = Vec::with_capacity(6 + remaining_accounts.len());
        account_infos.push(self.__program.clone());
        account_infos.push(self.authority.0.clone());
        account_infos.push(self.permissioned_account.0.clone());
        account_infos.push(self.permission.clone());
        account_infos.push(self.magic_program.clone());
        account_infos.push(self.magic_context.clone());
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

/// compat::Instruction builder for `CommitAndUndelegatePermission` via CPI.
///
/// ### Accounts (magic_program and magic_context are auto-set):
///
///   0. `[signer?]` authority - Either this or permissioned_account must be a signer
///   1. `[writable, signer?]` permissioned_account - Either this or authority must be a signer
///   2. `[writable]` permission
///   3. `[]` magic_program (auto-set)
///   4. `[writable]` magic_context (auto-set)
#[derive(Clone, Debug)]
pub struct CommitAndUndelegatePermissionCpiBuilder<'a, 'b> {
    instruction: Box<CommitAndUndelegatePermissionCpiBuilderInstruction<'a, 'b>>,
}

impl<'a, 'b> CommitAndUndelegatePermissionCpiBuilder<'a, 'b> {
    pub fn new(program: &'b compat::AccountInfo<'a>) -> Self {
        let instruction = Box::new(CommitAndUndelegatePermissionCpiBuilderInstruction {
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
    pub fn authority(
        &mut self,
        authority: &'b compat::AccountInfo<'a>,
        as_signer: bool,
    ) -> &mut Self {
        self.instruction.authority = Some((authority, as_signer));
        self
    }
    #[inline(always)]
    pub fn permissioned_account(
        &mut self,
        permissioned_account: &'b compat::AccountInfo<'a>,
        as_signer: bool,
    ) -> &mut Self {
        self.instruction.permissioned_account = Some((permissioned_account, as_signer));
        self
    }
    #[inline(always)]
    pub fn permission(&mut self, permission: &'b compat::AccountInfo<'a>) -> &mut Self {
        self.instruction.permission = Some(permission);
        self
    }
    #[inline(always)]
    pub fn magic_program(&mut self, magic_program: &'b compat::AccountInfo<'a>) -> &mut Self {
        self.instruction.magic_program = Some(magic_program);
        self
    }
    #[inline(always)]
    pub fn magic_context(&mut self, magic_context: &'b compat::AccountInfo<'a>) -> &mut Self {
        self.instruction.magic_context = Some(magic_context);
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
        let instruction = CommitAndUndelegatePermissionCpi {
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
        instruction.invoke_signed_with_remaining_accounts(
            signers_seeds,
            &self.instruction.__remaining_accounts,
        )
    }
}

#[derive(Clone, Debug)]
struct CommitAndUndelegatePermissionCpiBuilderInstruction<'a, 'b> {
    __program: &'b compat::AccountInfo<'a>,
    authority: Option<(&'b compat::AccountInfo<'a>, bool)>,
    permissioned_account: Option<(&'b compat::AccountInfo<'a>, bool)>,
    permission: Option<&'b compat::AccountInfo<'a>>,
    magic_program: Option<&'b compat::AccountInfo<'a>>,
    magic_context: Option<&'b compat::AccountInfo<'a>>,
    /// Additional instruction accounts `(compat::AccountInfo, is_writable, is_signer)`.
    __remaining_accounts: Vec<(&'b compat::AccountInfo<'a>, bool, bool)>,
}
