
use borsh::{BorshDeserialize, BorshSerialize};

use crate::access_control::programs::MAGICBLOCK_PERMISSION_API_ID;
use crate::consts::{MAGIC_CONTEXT_ID, MAGIC_PROGRAM_ID};
use crate::solana_compat::solana::{
    invoke, invoke_signed, AccountInfo, AccountMeta, Instruction, ProgramResult, Pubkey,
};

pub const COMMIT_AND_UNDELEGATE_PERMISSION_DISCRIMINATOR: u64 = 5;

/// Accounts.
#[derive(Debug)]
pub struct CommitAndUndelegatePermission {
    pub authority: (Pubkey, bool),

    pub permissioned_account: (Pubkey, bool),

    pub permission: Pubkey,

    pub magic_program: Pubkey,

    pub magic_context: Pubkey,
}

impl CommitAndUndelegatePermission {
    pub fn instruction(&self) -> Instruction {
        self.instruction_with_remaining_accounts(&[])
    }
    #[allow(clippy::arithmetic_side_effects)]
    #[allow(clippy::vec_init_then_push)]
    pub fn instruction_with_remaining_accounts(
        &self,
        remaining_accounts: &[AccountMeta],
    ) -> Instruction {
        let mut accounts = Vec::with_capacity(5 + remaining_accounts.len());
        accounts.push(AccountMeta::new_readonly(
            self.authority.0,
            self.authority.1,
        ));
        accounts.push(AccountMeta::new(
            self.permissioned_account.0,
            self.permissioned_account.1,
        ));
        accounts.push(AccountMeta::new(self.permission, false));
        accounts.push(AccountMeta::new_readonly(self.magic_program, false));
        accounts.push(AccountMeta::new(self.magic_context, false));
        accounts.extend_from_slice(remaining_accounts);
        let data = CommitAndUndelegatePermissionInstructionData::new()
            .try_to_vec()
            .unwrap();

        Instruction {
            program_id: MAGICBLOCK_PERMISSION_API_ID,
            accounts,
            data,
        }
    }
}

#[derive(BorshSerialize, BorshDeserialize, Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct CommitAndUndelegatePermissionInstructionData {
    discriminator: u64,
}

impl CommitAndUndelegatePermissionInstructionData {
    pub fn new() -> Self {
        Self { discriminator: 5 }
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

/// Instruction builder for `CommitAndUndelegatePermission`.
///
/// ### Accounts (magic_program and magic_context are auto-set):
///
///   0. `[signer]` authority
///   1. `[writable, signer]` permissioned_account
///   2. `[writable]` permission
///   3. `[]` magic_program (auto-set to MAGIC_PROGRAM_ID)
///   4. `[writable]` magic_context (auto-set to MAGIC_CONTEXT_ID)
#[derive(Clone, Debug, Default)]
pub struct CommitAndUndelegatePermissionBuilder {
    authority: Option<(Pubkey, bool)>,
    permissioned_account: Option<(Pubkey, bool)>,
    permission: Option<Pubkey>,
    __remaining_accounts: Vec<AccountMeta>,
}

impl CommitAndUndelegatePermissionBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a builder with the required accounts
    /// Magic program and magic context are auto-set
    pub fn with_accounts(
        authority: Pubkey,
        authority_is_signer: bool,
        permissioned_account: Pubkey,
        permissioned_is_signer: bool,
        permission: Pubkey,
    ) -> Self {
        Self {
            authority: Some((authority, authority_is_signer)),
            permissioned_account: Some((permissioned_account, permissioned_is_signer)),
            permission: Some(permission),
            __remaining_accounts: Vec::new(),
        }
    }

    #[inline(always)]
    pub fn authority(&mut self, authority: Pubkey, as_signer: bool) -> &mut Self {
        self.authority = Some((authority, as_signer));
        self
    }
    #[inline(always)]
    pub fn permissioned_account(
        &mut self,
        permissioned_account: Pubkey,
        as_signer: bool,
    ) -> &mut Self {
        self.permissioned_account = Some((permissioned_account, as_signer));
        self
    }
    #[inline(always)]
    pub fn permission(&mut self, permission: Pubkey) -> &mut Self {
        self.permission = Some(permission);
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
    pub authority: (&'b AccountInfo<'a>, bool),

    pub permissioned_account: (&'b AccountInfo<'a>, bool),

    pub permission: &'b AccountInfo<'a>,

    pub magic_program: &'b AccountInfo<'a>,

    pub magic_context: &'b AccountInfo<'a>,
}

/// `commit_and_undelegate_permission` CPI instruction.
pub struct CommitAndUndelegatePermissionCpi<'a, 'b> {
    /// The program to invoke.
    pub __program: &'b AccountInfo<'a>,

    pub authority: (&'b AccountInfo<'a>, bool),

    pub permissioned_account: (&'b AccountInfo<'a>, bool),

    pub permission: &'b AccountInfo<'a>,

    pub magic_program: &'b AccountInfo<'a>,

    pub magic_context: &'b AccountInfo<'a>,
}

impl<'a, 'b> CommitAndUndelegatePermissionCpi<'a, 'b> {
    pub fn new(
        program: &'b AccountInfo<'a>,
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
        let mut accounts = Vec::with_capacity(5 + remaining_accounts.len());
        accounts.push(AccountMeta::new_readonly(
            *self.authority.0.key,
            self.authority.1,
        ));
        accounts.push(AccountMeta::new(
            *self.permissioned_account.0.key,
            self.permissioned_account.1,
        ));
        accounts.push(AccountMeta::new(*self.permission.key, false));
        accounts.push(AccountMeta::new_readonly(*self.magic_program.key, false));
        accounts.push(AccountMeta::new(*self.magic_context.key, false));
        remaining_accounts.iter().for_each(|remaining_account| {
            accounts.push(AccountMeta {
                pubkey: *remaining_account.0.key,
                is_signer: remaining_account.1,
                is_writable: remaining_account.2,
            })
        });
        let data = CommitAndUndelegatePermissionInstructionData::new()
            .try_to_vec()
            .unwrap();

        let instruction = Instruction {
            program_id: MAGICBLOCK_PERMISSION_API_ID,
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
            invoke(&instruction, &account_infos)
        } else {
            invoke_signed(&instruction, &account_infos, signers_seeds)
        }
    }
}

/// Instruction builder for `CommitAndUndelegatePermission` via CPI.
///
/// ### Accounts (magic_program and magic_context are auto-set):
///
///   0. `[signer]` authority
///   1. `[writable, signer]` permissioned_account
///   2. `[writable]` permission
///   3. `[]` magic_program (auto-set)
///   4. `[writable]` magic_context (auto-set)
#[derive(Clone, Debug)]
pub struct CommitAndUndelegatePermissionCpiBuilder<'a, 'b> {
    instruction: Box<CommitAndUndelegatePermissionCpiBuilderInstruction<'a, 'b>>,
}

impl<'a, 'b> CommitAndUndelegatePermissionCpiBuilder<'a, 'b> {
    pub fn new(program: &'b AccountInfo<'a>) -> Self {
        let instruction = Box::new(CommitAndUndelegatePermissionCpiBuilderInstruction {
            __program: program,
            authority: None,
            permissioned_account: None,
            permission: None,
            __remaining_accounts: Vec::new(),
        });
        Self { instruction }
    }

    /// Create a CPI builder with the required accounts
    /// Magic program and magic context are auto-set
    pub fn with_accounts(
        program: &'b AccountInfo<'a>,
        authority: &'b AccountInfo<'a>,
        authority_is_signer: bool,
        permissioned_account: &'b AccountInfo<'a>,
        permissioned_is_signer: bool,
        permission: &'b AccountInfo<'a>,
    ) -> Self {
        let instruction = Box::new(CommitAndUndelegatePermissionCpiBuilderInstruction {
            __program: program,
            authority: Some((authority, authority_is_signer)),
            permissioned_account: Some((permissioned_account, permissioned_is_signer)),
            permission: Some(permission),
            __remaining_accounts: Vec::new(),
        });
        Self { instruction }
    }

    #[inline(always)]
    pub fn authority(&mut self, authority: &'b AccountInfo<'a>, as_signer: bool) -> &mut Self {
        self.instruction.authority = Some((authority, as_signer));
        self
    }
    #[inline(always)]
    pub fn permissioned_account(
        &mut self,
        permissioned_account: &'b AccountInfo<'a>,
        as_signer: bool,
    ) -> &mut Self {
        self.instruction.permissioned_account = Some((permissioned_account, as_signer));
        self
    }
    #[inline(always)]
    pub fn permission(&mut self, permission: &'b AccountInfo<'a>) -> &mut Self {
        self.instruction.permission = Some(permission);
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
        // Create magic_program and magic_context AccountInfos from constants
        // These should be passed via remaining accounts in actual usage
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
                .__remaining_accounts
                .iter()
                .find(|(acc, _, _)| acc.key == &MAGIC_PROGRAM_ID)
                .map(|(acc, _, _)| acc)
                .expect("magic_program account not found in remaining accounts"),

            magic_context: self
                .instruction
                .__remaining_accounts
                .iter()
                .find(|(acc, _, _)| acc.key == &MAGIC_CONTEXT_ID)
                .map(|(acc, _, _)| acc)
                .expect("magic_context account not found in remaining accounts"),
        };
        instruction.invoke_signed_with_remaining_accounts(
            signers_seeds,
            &self.instruction.__remaining_accounts,
        )
    }
}

#[derive(Clone, Debug)]
struct CommitAndUndelegatePermissionCpiBuilderInstruction<'a, 'b> {
    __program: &'b AccountInfo<'a>,
    authority: Option<(&'b AccountInfo<'a>, bool)>,
    permissioned_account: Option<(&'b AccountInfo<'a>, bool)>,
    permission: Option<&'b AccountInfo<'a>>,
    /// Additional instruction accounts `(AccountInfo, is_writable, is_signer)`.
    __remaining_accounts: Vec<(&'b AccountInfo<'a>, bool, bool)>,
}
