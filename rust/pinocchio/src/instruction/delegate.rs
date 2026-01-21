use pinocchio::{
    cpi::{Seed, Signer},
    error::ProgramError,
    AccountView, Address, ProgramResult,
};
use pinocchio_system::instructions::{Assign, CreateAccount};

use crate::consts::DELEGATION_PROGRAM_ID;
use crate::pda::find_program_address;
use crate::types::DelegateAccountArgs;
use crate::utils::{cpi_delegate, make_seed_buf};
use crate::{consts::BUFFER, types::DelegateConfig, utils::close_pda_acc};

/// Find the bump for a buffer PDA using the pinocchio PDA derivation.
fn find_buffer_pda_bump(pda_key: &[u8], owner_program: &Address) -> u8 {
    let (_, bump) = find_program_address(&[BUFFER, pda_key], owner_program);
    bump
}

#[allow(unknown_lints, clippy::cloned_ref_to_slice_refs)]
pub fn delegate_account(
    accounts: &[&AccountView],
    seeds: &[&[u8]],
    bump: u8,
    config: DelegateConfig,
) -> ProgramResult {
    let [payer, pda_acc, owner_program, buffer_acc, delegation_record, delegation_metadata, system_program] =
        accounts
    else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };

    if !payer.is_signer() {
        return Err(ProgramError::MissingRequiredSignature);
    }

    // Buffer PDA seeds
    let pda_key_bytes: &[u8; 32] = pda_acc.address().as_array();

    // Find buffer PDA bump
    let buffer_pda_bump = find_buffer_pda_bump(pda_key_bytes.as_ref(), owner_program.address());

    // Buffer signer seeds
    let buffer_bump_slice = [buffer_pda_bump];
    let buffer_seed_binding = [
        Seed::from(BUFFER),
        Seed::from(pda_key_bytes.as_ref()),
        Seed::from(&buffer_bump_slice),
    ];
    let buffer_signer_seeds = Signer::from(&buffer_seed_binding);

    // Single data_len and rent lookup
    let data_len = pda_acc.data_len();

    // Create Buffer PDA
    CreateAccount {
        from: payer,
        to: buffer_acc,
        lamports: 0,
        space: data_len as u64,
        owner: owner_program.address(),
    }
    .invoke_signed(&[buffer_signer_seeds])?;

    // Copy delegated PDA -> buffer, then zero delegated PDA
    {
        let pda_ro = pda_acc.try_borrow()?;
        let mut buf_data = buffer_acc.try_borrow_mut()?;
        buf_data.copy_from_slice(&pda_ro);
    }
    {
        let mut pda_mut = pda_acc.try_borrow_mut()?;
        for b in pda_mut.iter_mut().take(data_len) {
            *b = 0;
        }
    }

    // Assign delegated PDA to system if needed, then to delegation program
    let mut seed_buf = make_seed_buf();
    let filled = fill_seeds(&mut seed_buf, seeds, &bump);
    let delegate_signer_seeds = Signer::from(filled);

    let current_owner = unsafe { pda_acc.owner() };
    if current_owner != &pinocchio_system::ID {
        unsafe { pda_acc.assign(&pinocchio_system::ID) };
    }
    let current_owner = unsafe { pda_acc.owner() };
    if current_owner != &DELEGATION_PROGRAM_ID {
        Assign {
            account: pda_acc,
            owner: &DELEGATION_PROGRAM_ID,
        }
        .invoke_signed(&[delegate_signer_seeds.clone()])?;
    }

    // Delegate
    let delegate_args = DelegateAccountArgs {
        commit_frequency_ms: config.commit_frequency_ms,
        seeds,
        validator: config.validator,
    };

    cpi_delegate(
        payer,
        pda_acc,
        owner_program,
        buffer_acc,
        delegation_record,
        delegation_metadata,
        system_program,
        delegate_args,
        delegate_signer_seeds,
    )?;

    // Close buffer PDA back to payer to reclaim lamports
    close_pda_acc(payer, buffer_acc)?;

    Ok(())
}

pub struct DelegateAccountCpiBuilder<'a> {
    payer: &'a AccountView,
    pda_acc: &'a AccountView,
    owner_program: &'a AccountView,
    buffer_acc: &'a AccountView,
    delegation_record: &'a AccountView,
    delegation_metadata: &'a AccountView,
    system_program: &'a AccountView,
    seeds: Option<&'a [&'a [u8]]>,
    bump: Option<u8>,
    config: Option<DelegateConfig>,
}

impl<'a> DelegateAccountCpiBuilder<'a> {
    pub fn new(
        payer: &'a AccountView,
        pda_acc: &'a AccountView,
        owner_program: &'a AccountView,
        buffer_acc: &'a AccountView,
        delegation_record: &'a AccountView,
        delegation_metadata: &'a AccountView,
        system_program: &'a AccountView,
    ) -> Self {
        Self {
            payer,
            pda_acc,
            owner_program,
            buffer_acc,
            delegation_record,
            delegation_metadata,
            system_program,
            seeds: None,
            bump: None,
            config: None,
        }
    }

    pub fn seeds(mut self, seeds: &'a [&'a [u8]]) -> Self {
        self.seeds = Some(seeds);
        self
    }

    pub fn bump(mut self, bump: u8) -> Self {
        self.bump = Some(bump);
        self
    }

    pub fn config(mut self, config: DelegateConfig) -> Self {
        self.config = Some(config);
        self
    }

    pub fn invoke(self) -> ProgramResult {
        let seeds = self.seeds.ok_or(ProgramError::InvalidInstructionData)?;
        if seeds.len() > 15 {
            return Err(ProgramError::InvalidInstructionData);
        }
        let bump = self.bump.ok_or(ProgramError::InvalidInstructionData)?;
        let config = self.config.ok_or(ProgramError::InvalidInstructionData)?;
        delegate_account(
            &[
                self.payer,
                self.pda_acc,
                self.owner_program,
                self.buffer_acc,
                self.delegation_record,
                self.delegation_metadata,
                self.system_program,
            ],
            seeds,
            bump,
            config,
        )
    }
}

pub fn fill_seeds<'a>(
    out: &'a mut [Seed<'a>; 16],
    seeds: &[&'a [u8]],
    bump_ref: &'a u8,
) -> &'a [Seed<'a>] {
    assert!(seeds.len() <= 15, "too many seeds (max 15 + bump = 16)");

    let bump_slice: &[u8] = core::slice::from_ref(bump_ref);

    let mut i = 0;
    while i < seeds.len() {
        out[i] = Seed::from(seeds[i]);
        i += 1;
    }
    out[i] = Seed::from(bump_slice);

    &out[..=i]
}
