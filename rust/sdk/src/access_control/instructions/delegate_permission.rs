use borsh::{BorshDeserialize, BorshSerialize};

use crate::consts::PERMISSION_PROGRAM_ID;
use crate::solana_compat::solana::{
    invoke, invoke_signed, AccountInfo, AccountMeta, Instruction, ProgramResult, Pubkey,
};

pub const DELEGATE_PERMISSION_DISCRIMINATOR: u64 = 3;

/// Accounts.
#[derive(Debug)]
pub struct DelegatePermission {
    pub payer: Pubkey,

    pub authority: (Pubkey, bool),

    pub permissioned_account: (Pubkey, bool),

    pub permission: Pubkey,

    pub system_program: Pubkey,

    pub owner_program: Pubkey,

    pub delegation_buffer: Pubkey,

    pub delegation_record: Pubkey,

    pub delegation_metadata: Pubkey,

    pub delegation_program: Pubkey,

    pub validator: Pubkey,
}

impl DelegatePermission {
    pub fn instruction(&self) -> Instruction {
        self.instruction_with_remaining_accounts(&[])
    }
    #[allow(clippy::arithmetic_side_effects)]
    #[allow(clippy::vec_init_then_push)]
    pub fn instruction_with_remaining_accounts(
        &self,
        remaining_accounts: &[AccountMeta],
    ) -> Instruction {
        let mut accounts = Vec::with_capacity(11 + remaining_accounts.len());
        accounts.push(AccountMeta::new(self.payer, true));
        accounts.push(AccountMeta::new_readonly(
            self.authority.0,
            self.authority.1,
        ));
        accounts.push(AccountMeta::new_readonly(
            self.permissioned_account.0,
            self.permissioned_account.1,
        ));
        accounts.push(AccountMeta::new(self.permission, false));
        accounts.push(AccountMeta::new_readonly(self.system_program, false));
        accounts.push(AccountMeta::new_readonly(self.owner_program, false));
        accounts.push(AccountMeta::new(self.delegation_buffer, false));
        accounts.push(AccountMeta::new(self.delegation_record, false));
        accounts.push(AccountMeta::new(self.delegation_metadata, false));
        accounts.push(AccountMeta::new_readonly(self.delegation_program, false));
        accounts.push(AccountMeta::new_readonly(self.validator, false));
        accounts.extend_from_slice(remaining_accounts);
        let data = DelegatePermissionInstructionData::new()
            .try_to_vec()
            .expect("failed to serialize DelegatePermissionInstructionData");

        Instruction {
            program_id: PERMISSION_PROGRAM_ID,
            accounts,
            data,
        }
    }
}

#[derive(BorshSerialize, BorshDeserialize, Clone, Debug, Eq, PartialEq)]
pub struct DelegatePermissionInstructionData {
    discriminator: u64,
}

impl DelegatePermissionInstructionData {
    pub fn new() -> Self {
        Self {
            discriminator: DELEGATE_PERMISSION_DISCRIMINATOR,
        }
    }

    pub(crate) fn try_to_vec(&self) -> Result<Vec<u8>, std::io::Error> {
        borsh::to_vec(self)
    }
}

impl Default for DelegatePermissionInstructionData {
    fn default() -> Self {
        Self::new()
    }
}

/// Instruction builder for `DelegatePermission`.
///
/// ### Accounts (auto-derived from permissioned_account):
///
///   0. `[writable, signer]` payer
///   1. `[signer?]` authority - Either this or permissioned_account must be a signer
///   2. `[signer?]` permissioned_account - Either this or authority must be a signer
///   3. `[writable]` permission (auto-derived from permissioned_account)
///   4. `[]` system_program
///   5. `[]` owner_program (defaults to PERMISSION_PROGRAM_ID)
///   6. `[writable]` delegation_buffer (auto-derived from permission + permission_program)
///   7. `[writable]` delegation_record (auto-derived from permission)
///   8. `[writable]` delegation_metadata (auto-derived from permission)
///   9. `[]` delegation_program
///   10. `[optional]` validator
#[derive(Clone, Debug, Default)]
pub struct DelegatePermissionBuilder {
    payer: Option<Pubkey>,
    authority: Option<(Pubkey, bool)>,
    permissioned_account: Option<(Pubkey, bool)>,
    permission: Option<Pubkey>,
    system_program: Option<Pubkey>,
    owner_program: Option<Pubkey>,
    delegation_buffer: Option<Pubkey>,
    delegation_record: Option<Pubkey>,
    delegation_metadata: Option<Pubkey>,
    delegation_program: Option<Pubkey>,
    validator: Option<Pubkey>,
    __remaining_accounts: Vec<AccountMeta>,
}

impl DelegatePermissionBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    #[inline(always)]
    pub fn payer(&mut self, payer: Pubkey) -> &mut Self {
        self.payer = Some(payer);
        self
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
    #[inline(always)]
    pub fn system_program(&mut self, system_program: Pubkey) -> &mut Self {
        self.system_program = Some(system_program);
        self
    }
    #[inline(always)]
    pub fn owner_program(&mut self, owner_program: Pubkey) -> &mut Self {
        self.owner_program = Some(owner_program);
        self
    }
    #[inline(always)]
    pub fn delegation_buffer(&mut self, delegation_buffer: Pubkey) -> &mut Self {
        self.delegation_buffer = Some(delegation_buffer);
        self
    }
    #[inline(always)]
    pub fn delegation_record(&mut self, delegation_record: Pubkey) -> &mut Self {
        self.delegation_record = Some(delegation_record);
        self
    }
    #[inline(always)]
    pub fn delegation_metadata(&mut self, delegation_metadata: Pubkey) -> &mut Self {
        self.delegation_metadata = Some(delegation_metadata);
        self
    }
    #[inline(always)]
    pub fn delegation_program(&mut self, delegation_program: Pubkey) -> &mut Self {
        self.delegation_program = Some(delegation_program);
        self
    }
    /// `[optional account]`
    #[inline(always)]
    pub fn validator(&mut self, validator: Option<Pubkey>) -> &mut Self {
        self.validator = validator;
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
        let accounts = DelegatePermission {
            payer: self.payer.expect("payer is not set"),
            authority: self.authority.expect("authority is not set"),
            permissioned_account: self
                .permissioned_account
                .expect("permissioned_account is not set"),
            permission: self.permission.expect("permission is not set"),
            system_program: self.system_program.expect("system_program is not set"),
            owner_program: self.owner_program.expect("owner_program is not set"),
            delegation_buffer: self
                .delegation_buffer
                .expect("delegation_buffer is not set"),
            delegation_record: self
                .delegation_record
                .expect("delegation_record is not set"),
            delegation_metadata: self
                .delegation_metadata
                .expect("delegation_metadata is not set"),
            delegation_program: self
                .delegation_program
                .expect("delegation_program is not set"),
            validator: self.validator.expect("validator is not set"),
        };

        accounts.instruction_with_remaining_accounts(&self.__remaining_accounts)
    }
}

/// `delegate_permission` CPI accounts.
pub struct DelegatePermissionCpiAccounts<'a, 'b> {
    pub payer: &'b AccountInfo<'a>,

    pub authority: (&'b AccountInfo<'a>, bool),

    pub permissioned_account: (&'b AccountInfo<'a>, bool),

    pub permission: &'b AccountInfo<'a>,

    pub system_program: &'b AccountInfo<'a>,

    pub owner_program: &'b AccountInfo<'a>,

    pub delegation_buffer: &'b AccountInfo<'a>,

    pub delegation_record: &'b AccountInfo<'a>,

    pub delegation_metadata: &'b AccountInfo<'a>,

    pub delegation_program: &'b AccountInfo<'a>,

    pub validator: Option<&'b AccountInfo<'a>>,
}

/// `delegate_permission` CPI instruction.
pub struct DelegatePermissionCpi<'a, 'b> {
    /// The program to invoke.
    pub __program: &'b AccountInfo<'a>,

    pub payer: &'b AccountInfo<'a>,

    pub authority: (&'b AccountInfo<'a>, bool),

    pub permissioned_account: (&'b AccountInfo<'a>, bool),

    pub permission: &'b AccountInfo<'a>,

    pub system_program: &'b AccountInfo<'a>,

    pub owner_program: &'b AccountInfo<'a>,

    pub delegation_buffer: &'b AccountInfo<'a>,

    pub delegation_record: &'b AccountInfo<'a>,

    pub delegation_metadata: &'b AccountInfo<'a>,

    pub delegation_program: &'b AccountInfo<'a>,

    pub validator: Option<&'b AccountInfo<'a>>,
}

impl<'a, 'b> DelegatePermissionCpi<'a, 'b> {
    pub fn new(
        program: &'b AccountInfo<'a>,
        accounts: DelegatePermissionCpiAccounts<'a, 'b>,
    ) -> Self {
        Self {
            __program: program,
            payer: accounts.payer,
            authority: accounts.authority,
            permissioned_account: accounts.permissioned_account,
            permission: accounts.permission,
            system_program: accounts.system_program,
            owner_program: accounts.owner_program,
            delegation_buffer: accounts.delegation_buffer,
            delegation_record: accounts.delegation_record,
            delegation_metadata: accounts.delegation_metadata,
            delegation_program: accounts.delegation_program,
            validator: accounts.validator,
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
        let mut accounts = Vec::with_capacity(11 + remaining_accounts.len());
        accounts.push(AccountMeta::new(*self.payer.key, true));
        accounts.push(AccountMeta::new_readonly(
            *self.authority.0.key,
            self.authority.1,
        ));
        accounts.push(AccountMeta::new_readonly(
            *self.permissioned_account.0.key,
            self.permissioned_account.1,
        ));
        accounts.push(AccountMeta::new(*self.permission.key, false));
        accounts.push(AccountMeta::new_readonly(*self.system_program.key, false));
        accounts.push(AccountMeta::new_readonly(*self.owner_program.key, false));
        accounts.push(AccountMeta::new(*self.delegation_buffer.key, false));
        accounts.push(AccountMeta::new(*self.delegation_record.key, false));
        accounts.push(AccountMeta::new(*self.delegation_metadata.key, false));
        accounts.push(AccountMeta::new_readonly(
            *self.delegation_program.key,
            false,
        ));
        if let Some(validator) = self.validator {
            accounts.push(AccountMeta::new_readonly(*validator.key, false));
        }
        remaining_accounts.iter().for_each(|remaining_account| {
            accounts.push(AccountMeta {
                pubkey: *remaining_account.0.key,
                is_signer: remaining_account.1,
                is_writable: remaining_account.2,
            })
        });
        let data = DelegatePermissionInstructionData::new()
            .try_to_vec()
            .expect("failed to serialize DelegatePermissionInstructionData");

        let instruction = Instruction {
            program_id: PERMISSION_PROGRAM_ID,
            accounts,
            data,
        };
        let mut account_infos = Vec::with_capacity(12 + remaining_accounts.len());
        account_infos.push(self.__program.clone());
        account_infos.push(self.payer.clone());
        account_infos.push(self.authority.0.clone());
        account_infos.push(self.permissioned_account.0.clone());
        account_infos.push(self.permission.clone());
        account_infos.push(self.system_program.clone());
        account_infos.push(self.owner_program.clone());
        account_infos.push(self.delegation_buffer.clone());
        account_infos.push(self.delegation_record.clone());
        account_infos.push(self.delegation_metadata.clone());
        account_infos.push(self.delegation_program.clone());
        if let Some(validator) = self.validator {
            account_infos.push(validator.clone());
        }
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

/// Instruction builder for `DelegatePermission` via CPI.
///
/// ### Accounts:
///
///   0. `[writable, signer]` payer
///   1. `[signer?]` authority - Either this or permissioned_account must be a signer
///   2. `[signer?]` permissioned_account - Either this or authority must be a signer
///   3. `[writable]` permission
///   4. `[]` system_program
///   5. `[]` owner_program
///   6. `[writable]` delegation_buffer
///   7. `[writable]` delegation_record
///   8. `[writable]` delegation_metadata
///   9. `[]` delegation_program
///   10. `[optional]` validator
#[derive(Clone, Debug)]
pub struct DelegatePermissionCpiBuilder<'a, 'b> {
    instruction: Box<DelegatePermissionCpiBuilderInstruction<'a, 'b>>,
}

impl<'a, 'b> DelegatePermissionCpiBuilder<'a, 'b> {
    /// Create a new delegate permission CPI builder.
    ///
    /// Optionally accepts a `permission` AccountInfo. All other accounts must be set
    /// via their respective builder methods (payer, authority, permissioned_account, etc.)
    pub fn new(program: &'b AccountInfo<'a>) -> Self {
        let instruction = Box::new(DelegatePermissionCpiBuilderInstruction {
            __program: program,
            payer: None,
            authority: None,
            permissioned_account: None,
            permission: None,
            system_program: None,
            owner_program: None,
            delegation_buffer: None,
            delegation_record: None,
            delegation_metadata: None,
            delegation_program: None,
            validator: None,
            __remaining_accounts: Vec::new(),
        });
        Self { instruction }
    }
    #[inline(always)]
    pub fn payer(&mut self, payer: &'b AccountInfo<'a>) -> &mut Self {
        self.instruction.payer = Some(payer);
        self
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
    #[inline(always)]
    pub fn system_program(&mut self, system_program: &'b AccountInfo<'a>) -> &mut Self {
        self.instruction.system_program = Some(system_program);
        self
    }
    #[inline(always)]
    pub fn owner_program(&mut self, owner_program: &'b AccountInfo<'a>) -> &mut Self {
        self.instruction.owner_program = Some(owner_program);
        self
    }
    #[inline(always)]
    pub fn delegation_buffer(&mut self, delegation_buffer: &'b AccountInfo<'a>) -> &mut Self {
        self.instruction.delegation_buffer = Some(delegation_buffer);
        self
    }
    #[inline(always)]
    pub fn delegation_record(&mut self, delegation_record: &'b AccountInfo<'a>) -> &mut Self {
        self.instruction.delegation_record = Some(delegation_record);
        self
    }
    #[inline(always)]
    pub fn delegation_metadata(&mut self, delegation_metadata: &'b AccountInfo<'a>) -> &mut Self {
        self.instruction.delegation_metadata = Some(delegation_metadata);
        self
    }
    #[inline(always)]
    pub fn delegation_program(&mut self, delegation_program: &'b AccountInfo<'a>) -> &mut Self {
        self.instruction.delegation_program = Some(delegation_program);
        self
    }
    /// `[optional account]`
    #[inline(always)]
    pub fn validator(&mut self, validator: Option<&'b AccountInfo<'a>>) -> &mut Self {
        self.instruction.validator = validator;
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
        let instruction = DelegatePermissionCpi {
            __program: self.instruction.__program,

            payer: self.instruction.payer.expect("payer is not set"),

            authority: self.instruction.authority.expect("authority is not set"),

            permissioned_account: self
                .instruction
                .permissioned_account
                .expect("permissioned_account is not set"),

            permission: self.instruction.permission.expect("permission is not set"),

            system_program: self
                .instruction
                .system_program
                .expect("system_program is not set"),

            owner_program: self
                .instruction
                .owner_program
                .expect("owner_program is not set"),

            delegation_buffer: self
                .instruction
                .delegation_buffer
                .expect("delegation_buffer is not set"),

            delegation_record: self
                .instruction
                .delegation_record
                .expect("delegation_record is not set"),

            delegation_metadata: self
                .instruction
                .delegation_metadata
                .expect("delegation_metadata is not set"),

            delegation_program: self
                .instruction
                .delegation_program
                .expect("delegation_program is not set"),

            validator: Some(self.instruction.validator.expect("validator is not set")),
        };
        instruction.invoke_signed_with_remaining_accounts(
            signers_seeds,
            &self.instruction.__remaining_accounts,
        )
    }
}

#[derive(Clone, Debug)]
struct DelegatePermissionCpiBuilderInstruction<'a, 'b> {
    __program: &'b AccountInfo<'a>,
    payer: Option<&'b AccountInfo<'a>>,
    authority: Option<(&'b AccountInfo<'a>, bool)>,
    permissioned_account: Option<(&'b AccountInfo<'a>, bool)>,
    permission: Option<&'b AccountInfo<'a>>,
    system_program: Option<&'b AccountInfo<'a>>,
    owner_program: Option<&'b AccountInfo<'a>>,
    delegation_buffer: Option<&'b AccountInfo<'a>>,
    delegation_record: Option<&'b AccountInfo<'a>>,
    delegation_metadata: Option<&'b AccountInfo<'a>>,
    delegation_program: Option<&'b AccountInfo<'a>>,
    validator: Option<&'b AccountInfo<'a>>,
    /// Additional instruction accounts `(AccountInfo, is_writable, is_signer)`.
    __remaining_accounts: Vec<(&'b AccountInfo<'a>, bool, bool)>,
}
