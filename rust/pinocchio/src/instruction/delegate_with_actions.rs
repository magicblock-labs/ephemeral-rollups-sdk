use pinocchio::{cpi::Signer, error::ProgramError, AccountView, ProgramResult};

use crate::types::PostDelegationActions;
use crate::utils::cpi_delegate_with_actions;
use crate::utils::{fill_seeds, make_seed_buf};
use crate::{types::DelegateAccountArgs, utils::cpi_delegate_prepare};
use crate::{types::DelegateConfig, utils::close_pda_acc};

#[allow(unknown_lints, clippy::cloned_ref_to_slice_refs)]
pub fn delegate_with_actions<'a>(
    accounts: &[&AccountView],
    seeds: &[&[u8]],
    bump: u8,
    config: DelegateConfig,
    actions: PostDelegationActions<'a>,
) -> ProgramResult {
    let [payer, pda_acc, owner_program, buffer_acc, delegation_record, delegation_metadata, system_program] =
        accounts
    else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };

    // Assign delegated PDA to system if needed, then to delegation program
    let mut seed_buf = make_seed_buf();
    let filled = fill_seeds(&mut seed_buf, seeds, &bump);
    let delegate_signer_seeds = Signer::from(filled);

    cpi_delegate_prepare(
        payer,
        pda_acc,
        owner_program,
        buffer_acc,
        &delegate_signer_seeds,
    )?;

    // Delegate
    let delegate_args = DelegateAccountArgs {
        commit_frequency_ms: config.commit_frequency_ms,
        seeds,
        validator: config.validator,
    };

    cpi_delegate_with_actions(
        payer,
        pda_acc,
        owner_program,
        buffer_acc,
        delegation_record,
        delegation_metadata,
        system_program,
        delegate_args,
        actions,
        delegate_signer_seeds,
    )?;

    // Close buffer PDA back to payer to reclaim lamports
    close_pda_acc(payer, buffer_acc)?;

    Ok(())
}

pub struct DelegateWithActionsCpiBuilder<'a> {
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
    actions: Option<PostDelegationActions<'a>>,
}

impl<'a> DelegateWithActionsCpiBuilder<'a> {
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
            actions: None,
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

    pub fn actions(mut self, actions: PostDelegationActions<'a>) -> Self {
        self.actions = Some(actions);
        self
    }

    pub fn invoke(self) -> ProgramResult {
        let seeds = self.seeds.ok_or(ProgramError::InvalidInstructionData)?;
        if seeds.len() > 15 {
            return Err(ProgramError::InvalidInstructionData);
        }
        let bump = self.bump.ok_or(ProgramError::InvalidInstructionData)?;
        let config = self.config.ok_or(ProgramError::InvalidInstructionData)?;
        let actions = self.actions.ok_or(ProgramError::InvalidInstructionData)?;
        delegate_with_actions(
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
            actions,
        )
    }
}
