/// Undelegate permission instruction builder
use crate::access_control::pinocchio::instructions::UNDELEGATE_PERMISSION_DISCRIMINATOR;
use crate::access_control::pinocchio::pda::permission_program_id;
use pinocchio::{instruction::InstructionAccount, error::ProgramError, Address, AccountView, cpi::invoke_with_slice};
use core::mem::MaybeUninit;

const MAX_SEEDS_SIZE: usize = 4 + (4 * 16 + 1024 * 16);
const MAX_INSTRUCTION_DATA: usize = 8 + MAX_SEEDS_SIZE;

/// Instruction builder for `UndelegatePermission`.
///
/// ### Accounts:
///   0. `[writable]` delegated_permission
///   1. `[writable]` delegation_buffer
///   2. `[signer]` validator
///   3. `[]` system_program
#[derive(Clone, Debug, Default)]
pub struct UndelegatePermissionBuilder {
    delegated_permission: Option<Address>,
    delegation_buffer: Option<Address>,
    validator: Option<Address>,
    system_program: Option<Address>,
    pda_seeds: Option<Vec<Vec<u8>>>,
}

impl UndelegatePermissionBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    #[inline(always)]
    pub fn delegated_permission(&mut self, delegated_permission: Address) -> &mut Self {
        self.delegated_permission = Some(delegated_permission);
        self
    }

    #[inline(always)]
    pub fn delegation_buffer(&mut self, delegation_buffer: Address) -> &mut Self {
        self.delegation_buffer = Some(delegation_buffer);
        self
    }

    #[inline(always)]
    pub fn validator(&mut self, validator: Address) -> &mut Self {
        self.validator = Some(validator);
        self
    }

    #[inline(always)]
    pub fn system_program(&mut self, system_program: Address) -> &mut Self {
        self.system_program = Some(system_program);
        self
    }

    #[inline(always)]
    pub fn pda_seeds(&mut self, pda_seeds: Option<Vec<Vec<u8>>>) -> &mut Self {
        self.pda_seeds = pda_seeds;
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
        let delegated_permission = self
            .delegated_permission
            .as_ref()
            .ok_or(ProgramError::InvalidArgument)?;
        let delegation_buffer = self
            .delegation_buffer
            .as_ref()
            .ok_or(ProgramError::InvalidArgument)?;
        let validator = self.validator.as_ref().ok_or(ProgramError::InvalidArgument)?;
        let system_program = self
            .system_program
            .as_ref()
            .ok_or(ProgramError::InvalidArgument)?;

        let seeds_refs: Vec<&[u8]> = self
            .pda_seeds
            .as_ref()
            .map(|seeds| seeds.iter().map(|s| s.as_slice()).collect())
            .unwrap_or_default();

        undelegate_permission_instruction_impl(
            delegated_permission,
            delegation_buffer,
            validator,
            system_program,
            &seeds_refs,
            data_buf,
        )
    }
}

/// Build an undelegate_permission instruction
///
/// ### Accounts:
///   0. `[writable]` delegated_permission
///   1. `[writable]` delegation_buffer
///   2. `[signer]` validator
///   3. `[]` system_program
fn undelegate_permission_instruction_impl<'a>(
    delegated_permission: &Address,
    delegation_buffer: &Address,
    validator: &Address,
    system_program: &Address,
    pda_seeds: &[&[u8]],
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
            address: delegated_permission,
            is_writable: true,
            is_signer: false,
        });
        metas[1].write(InstructionAccount {
            address: delegation_buffer,
            is_writable: true,
            is_signer: false,
        });
        metas[2].write(InstructionAccount {
            address: validator,
            is_writable: false,
            is_signer: true,
        });
        metas[3].write(InstructionAccount {
            address: system_program,
            is_writable: false,
            is_signer: false,
        });
    }

    // Serialize discriminator
    let discriminator_bytes = UNDELEGATE_PERMISSION_DISCRIMINATOR.to_le_bytes();
    data_buf[0..8].copy_from_slice(&discriminator_bytes);

    // Serialize PDA seeds
    let seeds_len = serialize_seeds(&mut data_buf[8..], pda_seeds)?;
    let total_len = 8 + seeds_len;

    Ok((metas, 4, permission_program_id(), &data_buf[..total_len]))
}

fn serialize_seeds(buf: &mut [u8], seeds: &[&[u8]]) -> Result<usize, ProgramError> {
    if buf.len() < 4 {
        return Err(ProgramError::InvalidArgument);
    }

    if seeds.len() > 16 {
        return Err(ProgramError::InvalidArgument);
    }

    // Serialize count
    let count_bytes = (seeds.len() as u32).to_le_bytes();
    buf[0..4].copy_from_slice(&count_bytes);

    let mut offset = 4;

    // Serialize seeds
    for seed in seeds {
        if seed.len() > 1024 {
            return Err(ProgramError::InvalidArgument);
        }

        if offset + 4 + seed.len() > buf.len() {
            return Err(ProgramError::InvalidArgument);
        }

        // Seed length
        let len_bytes = (seed.len() as u32).to_le_bytes();
        buf[offset..offset + 4].copy_from_slice(&len_bytes);
        offset += 4;

        // Seed data
        buf[offset..offset + seed.len()].copy_from_slice(seed);
        offset += seed.len();
    }

    Ok(offset)
}

/// `undelegate_permission` CPI accounts using AccountView.
pub struct UndelegatePermissionCpiAccountsAccountView<'a> {
    pub delegated_permission: &'a AccountView,
    pub delegation_buffer: &'a AccountView,
    pub validator: &'a AccountView,
    pub system_program: &'a AccountView,
}

/// `undelegate_permission` CPI instruction using AccountView.
pub struct UndelegatePermissionCpiAccountView<'a> {
    /// The program to invoke.
    pub __program: &'a AccountView,
    pub delegated_permission: &'a AccountView,
    pub delegation_buffer: &'a AccountView,
    pub validator: &'a AccountView,
    pub system_program: &'a AccountView,
}

impl<'a> UndelegatePermissionCpiAccountView<'a> {
    pub fn new(
        program: &'a AccountView,
        accounts: UndelegatePermissionCpiAccountsAccountView<'a>,
    ) -> Self {
        Self {
            __program: program,
            delegated_permission: accounts.delegated_permission,
            delegation_buffer: accounts.delegation_buffer,
            validator: accounts.validator,
            system_program: accounts.system_program,
        }
    }

    #[inline(always)]
    pub fn invoke(&self, pda_seeds: Option<&[&[u8]]>) -> Result<(), ProgramError> {
        self.invoke_with_remaining_accounts(pda_seeds, &[])
    }

    #[inline(always)]
    pub fn invoke_with_remaining_accounts(
        &self,
        pda_seeds: Option<&[&[u8]]>,
        remaining_accounts: &[(&'a AccountView, bool, bool)],
    ) -> Result<(), ProgramError> {
        let mut data_buf = [0u8; MAX_INSTRUCTION_DATA];
        let (metas, num_accounts, program_id, data) = undelegate_permission_instruction_impl(
            &self.delegated_permission.address,
            &self.delegation_buffer.address,
            &self.validator.address,
            &self.system_program.address,
            pda_seeds.unwrap_or(&[]),
            &mut data_buf,
        )?;

        let mut all_accounts: [&AccountView; 8] = [self.__program; 8];
        all_accounts[0] = self.delegated_permission;
        all_accounts[1] = self.delegation_buffer;
        all_accounts[2] = self.validator;
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

/// Instruction builder for `UndelegatePermission` via CPI with AccountView.
///
/// ### Accounts:
///   0. `[writable]` delegated_permission
///   1. `[writable]` delegation_buffer
///   2. `[signer]` validator
///   3. `[]` system_program
#[derive(Clone, Debug)]
pub struct UndelegatePermissionCpiBuilderAccountView<'a> {
    instruction: Box<UndelegatePermissionCpiBuilderInstructionAccountView<'a>>,
}

impl<'a> UndelegatePermissionCpiBuilderAccountView<'a> {
    pub fn new(program: &'a AccountView) -> Self {
        let instruction = Box::new(UndelegatePermissionCpiBuilderInstructionAccountView {
            __program: program,
            delegated_permission: None,
            delegation_buffer: None,
            validator: None,
            system_program: None,
            pda_seeds: None,
            __remaining_accounts: Vec::new(),
        });
        Self { instruction }
    }

    #[inline(always)]
    pub fn delegated_permission(&mut self, delegated_permission: &'a AccountView) -> &mut Self {
        self.instruction.delegated_permission = Some(delegated_permission);
        self
    }

    #[inline(always)]
    pub fn delegation_buffer(&mut self, delegation_buffer: &'a AccountView) -> &mut Self {
        self.instruction.delegation_buffer = Some(delegation_buffer);
        self
    }

    #[inline(always)]
    pub fn validator(&mut self, validator: &'a AccountView) -> &mut Self {
        self.instruction.validator = Some(validator);
        self
    }

    #[inline(always)]
    pub fn system_program(&mut self, system_program: &'a AccountView) -> &mut Self {
        self.instruction.system_program = Some(system_program);
        self
    }

    #[inline(always)]
    pub fn pda_seeds(&mut self, pda_seeds: Option<Vec<Vec<u8>>>) -> &mut Self {
        self.instruction.pda_seeds = pda_seeds;
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
        let seeds_refs: Vec<&[u8]> = self
            .instruction
            .pda_seeds
            .as_ref()
            .map(|seeds| seeds.iter().map(|s| s.as_slice()).collect())
            .unwrap_or_default();

        let instruction = UndelegatePermissionCpiAccountView {
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
        };
        instruction.invoke_with_remaining_accounts(
            if seeds_refs.is_empty() { None } else { Some(&seeds_refs) },
            &self.instruction.__remaining_accounts,
        )
    }
}

#[derive(Clone, Debug)]
struct UndelegatePermissionCpiBuilderInstructionAccountView<'a> {
    __program: &'a AccountView,
    delegated_permission: Option<&'a AccountView>,
    delegation_buffer: Option<&'a AccountView>,
    validator: Option<&'a AccountView>,
    system_program: Option<&'a AccountView>,
    pda_seeds: Option<Vec<Vec<u8>>>,
    /// Additional instruction accounts `(AccountView, is_writable, is_signer)`.
    __remaining_accounts: Vec<(&'a AccountView, bool, bool)>,
}
