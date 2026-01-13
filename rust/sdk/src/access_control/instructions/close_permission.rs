use borsh::{BorshDeserialize, BorshSerialize};

use crate::access_control::structs::Permission;
use crate::consts::PERMISSION_PROGRAM_ID;
use crate::solana_compat::solana::{
    invoke, invoke_signed, AccountInfo, AccountMeta, Instruction, ProgramResult, Pubkey,
};

pub const CLOSE_PERMISSION_DISCRIMINATOR: u64 = 2;

/// Accounts.
#[derive(Debug)]
pub struct ClosePermission {
    pub payer: Pubkey,

    pub authority: (Pubkey, bool),

    pub permissioned_account: (Pubkey, bool),

    pub permission: Pubkey,
}

impl ClosePermission {
    pub fn instruction(&self) -> Instruction {
        self.instruction_with_remaining_accounts(&[])
    }
    #[allow(clippy::arithmetic_side_effects)]
    #[allow(clippy::vec_init_then_push)]
    pub fn instruction_with_remaining_accounts(
        &self,
        remaining_accounts: &[AccountMeta],
    ) -> Instruction {
        let mut accounts = Vec::with_capacity(4 + remaining_accounts.len());
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
        accounts.extend_from_slice(remaining_accounts);
        let data = ClosePermissionInstructionData::new().try_to_vec().unwrap();

        Instruction {
            program_id: PERMISSION_PROGRAM_ID,
            accounts,
            data,
        }
    }
}

#[derive(BorshSerialize, BorshDeserialize, Clone, Debug, Eq, PartialEq)]
pub struct ClosePermissionInstructionData {
    discriminator: u64,
}

impl ClosePermissionInstructionData {
    pub fn new() -> Self {
        Self {
            discriminator: CLOSE_PERMISSION_DISCRIMINATOR,
        }
    }

    pub(crate) fn try_to_vec(&self) -> Result<Vec<u8>, std::io::Error> {
        borsh::to_vec(self)
    }
}

impl Default for ClosePermissionInstructionData {
    fn default() -> Self {
        Self::new()
    }
}

/// Instruction builder for `ClosePermission`.
///
/// ### Accounts:
///
///   0. `[writable, signer]` payer
///   1. `[signer?]` authority - Either this or permissioned_account must be a signer
///   2. `[signer?]` permissioned_account - Either this or authority must be a signer
///   3. `[writable]` permission
#[derive(Clone, Debug, Default)]
pub struct ClosePermissionBuilder {
    payer: Option<Pubkey>,
    authority: Option<(Pubkey, bool)>,
    permissioned_account: Option<(Pubkey, bool)>,
    permission: Option<Pubkey>,
    __remaining_accounts: Vec<AccountMeta>,
}

impl ClosePermissionBuilder {
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
        let accounts = ClosePermission {
            payer: self.payer.expect("payer is not set"),
            authority: self.authority.expect("authority is not set"),
            permissioned_account: self
                .permissioned_account
                .expect("permissioned_account is not set"),
            permission: self.permission.expect("permission is not set"),
        };

        accounts.instruction_with_remaining_accounts(&self.__remaining_accounts)
    }
}

/// `close_permission` CPI accounts.
pub struct ClosePermissionCpiAccounts<'a, 'b> {
    pub payer: &'b AccountInfo<'a>,

    pub authority: (&'b AccountInfo<'a>, bool),

    pub permissioned_account: (&'b AccountInfo<'a>, bool),

    pub permission: &'b AccountInfo<'a>,
}

/// `close_permission` CPI instruction.
pub struct ClosePermissionCpi<'a, 'b> {
    /// The program to invoke.
    pub __program: &'b AccountInfo<'a>,

    pub payer: &'b AccountInfo<'a>,

    pub authority: (&'b AccountInfo<'a>, bool),

    pub permissioned_account: (&'b AccountInfo<'a>, bool),

    pub permission: &'b AccountInfo<'a>,
}

impl<'a, 'b> ClosePermissionCpi<'a, 'b> {
    pub fn new(program: &'b AccountInfo<'a>, accounts: ClosePermissionCpiAccounts<'a, 'b>) -> Self {
        Self {
            __program: program,
            payer: accounts.payer,
            authority: accounts.authority,
            permissioned_account: accounts.permissioned_account,
            permission: accounts.permission,
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
        remaining_accounts.iter().for_each(|remaining_account| {
            accounts.push(AccountMeta {
                pubkey: *remaining_account.0.key,
                is_signer: remaining_account.2,
                is_writable: remaining_account.1,
            })
        });
        let data = ClosePermissionInstructionData::new().try_to_vec().unwrap();

        let instruction = Instruction {
            program_id: PERMISSION_PROGRAM_ID,
            accounts,
            data,
        };
        let mut account_infos = Vec::with_capacity(4 + remaining_accounts.len());
        account_infos.push(self.__program.clone());
        account_infos.push(self.payer.clone());
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

/// Instruction builder for `ClosePermission` via CPI.
///
/// ### Accounts:
///
///   0. `[writable, signer]` payer
///   1. `[signer?]` authority - Either this or permissioned_account must be a signer
///   2. `[signer?]` permissioned_account - Either this or authority must be a signer
///   3. `[writable]` permission
#[derive(Clone, Debug)]
pub struct ClosePermissionCpiBuilder<'a, 'b> {
    instruction: Box<ClosePermissionCpiBuilderInstruction<'a, 'b>>,
}

impl<'a, 'b> ClosePermissionCpiBuilder<'a, 'b> {
    pub fn new(program: &'b AccountInfo<'a>) -> Self {
        let instruction = Box::new(ClosePermissionCpiBuilderInstruction {
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
        let instruction = ClosePermissionCpi {
            __program: self.instruction.__program,

            payer: self.instruction.payer.expect("payer is not set"),

            authority: self.instruction.authority.expect("authority is not set"),

            permissioned_account: self
                .instruction
                .permissioned_account
                .expect("permissioned_account is not set"),

            permission: self.instruction.permission.expect("permission is not set"),
        };
        instruction.invoke_signed_with_remaining_accounts(
            signers_seeds,
            &self.instruction.__remaining_accounts,
        )
    }
}

#[derive(Clone, Debug)]
struct ClosePermissionCpiBuilderInstruction<'a, 'b> {
    __program: &'b AccountInfo<'a>,
    payer: Option<&'b AccountInfo<'a>>,
    authority: Option<(&'b AccountInfo<'a>, bool)>,
    permissioned_account: Option<(&'b AccountInfo<'a>, bool)>,
    permission: Option<&'b AccountInfo<'a>>,
    /// Additional instruction accounts `(AccountInfo, is_writable, is_signer)`.
    __remaining_accounts: Vec<(&'b AccountInfo<'a>, bool, bool)>,
}
