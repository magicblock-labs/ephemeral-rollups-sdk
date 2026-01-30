use pinocchio::{cpi::Signer, error::ProgramError, AccountView, Address, ProgramResult};

use crate::acl::utils::cpi_close_permission;
use crate::utils::make_seed_buf;

/// Close a permission and recover rent.
pub fn close_permission(
    accounts: &[&AccountView],
    permission_program: &Address,
    authority_is_signer: bool,
    permissioned_account_is_signer: bool,
    signer_seeds: Option<Signer<'_, '_>>,
) -> ProgramResult {
    let [payer, authority, permissioned_account, permission] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };

    if !authority_is_signer && !permissioned_account_is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }

    cpi_close_permission(
        payer,
        authority,
        permissioned_account,
        permission,
        permission_program,
        authority_is_signer,
        permissioned_account_is_signer,
        signer_seeds,
    )
}

pub struct ClosePermissionCpiBuilder<'a> {
    payer: &'a AccountView,
    authority: &'a AccountView,
    permissioned_account: &'a AccountView,
    permission: &'a AccountView,
    permission_program: &'a Address,
    authority_is_signer: bool,
    permissioned_account_is_signer: bool,
    seeds: Option<&'a [&'a [u8]]>,
    bump: Option<u8>,
}

impl<'a> ClosePermissionCpiBuilder<'a> {
    pub fn new(
        payer: &'a AccountView,
        authority: &'a AccountView,
        permissioned_account: &'a AccountView,
        permission: &'a AccountView,
        permission_program: &'a Address,
    ) -> Self {
        Self {
            payer,
            authority,
            permissioned_account,
            permission,
            permission_program,
            authority_is_signer: true,
            permissioned_account_is_signer: true,
            seeds: None,
            bump: None,
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

    pub fn invoke(self) -> ProgramResult {
        let seeds = self.seeds.ok_or(ProgramError::InvalidInstructionData)?;
        let bump = self.bump.ok_or(ProgramError::InvalidInstructionData)?;

        let mut seed_buf = make_seed_buf();
        if seeds.len() + 1 > seed_buf.len() {
            return Err(ProgramError::InvalidInstructionData);
        }
        let bump_slice = [bump];
        for (i, seed) in seeds.iter().enumerate() {
            seed_buf[i] = pinocchio::cpi::Seed::from(*seed);
        }
        seed_buf[seeds.len()] = pinocchio::cpi::Seed::from(&bump_slice);
        let signer_seeds = Signer::from(&seed_buf[..=seeds.len()]);

        close_permission(
            &[
                self.payer,
                self.authority,
                self.permissioned_account,
                self.permission,
            ],
            self.permission_program,
            self.authority_is_signer,
            self.permissioned_account_is_signer,
            Some(signer_seeds),
        )
    }
}
