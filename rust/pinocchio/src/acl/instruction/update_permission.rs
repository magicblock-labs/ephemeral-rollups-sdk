use pinocchio::{cpi::Signer, error::ProgramError, AccountView, Address, ProgramResult};

use crate::acl::{types::MembersArgs, utils::cpi_update_permission};
use crate::utils::make_seed_buf;

/// Update a permission.
pub fn update_permission(
    accounts: &[&AccountView],
    permission_program: &Address,
    authority_is_signer: bool,
    permissioned_account_is_signer: bool,
    args: MembersArgs,
    signer_seeds: Option<Signer<'_, '_>>,
) -> ProgramResult {
    let [authority, permissioned_account, permission] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };

    if !authority_is_signer && !permissioned_account_is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }

    cpi_update_permission(
        authority,
        permissioned_account,
        permission,
        permission_program,
        authority_is_signer,
        permissioned_account_is_signer,
        args,
        signer_seeds,
    )
}

pub struct UpdatePermissionCpiBuilder<'a> {
    authority: &'a AccountView,
    permissioned_account: &'a AccountView,
    permission: &'a AccountView,
    permission_program: &'a Address,
    authority_is_signer: bool,
    permissioned_account_is_signer: bool,
    members: Option<MembersArgs<'a>>,
    seeds: Option<&'a [&'a [u8]]>,
    bump: Option<u8>,
}

impl<'a> UpdatePermissionCpiBuilder<'a> {
    pub fn new(
        authority: &'a AccountView,
        permissioned_account: &'a AccountView,
        permission: &'a AccountView,
        permission_program: &'a Address,
    ) -> Self {
        Self {
            authority,
            permissioned_account,
            permission,
            permission_program,
            authority_is_signer: true,
            permissioned_account_is_signer: true,
            members: None,
            seeds: None,
            bump: None,
        }
    }

    pub fn members(mut self, members: MembersArgs<'a>) -> Self {
        self.members = Some(members);
        self
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
        let members = self.members.ok_or(ProgramError::InvalidInstructionData)?;
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

        update_permission(
            &[self.authority, self.permissioned_account, self.permission],
            self.permission_program,
            self.authority_is_signer,
            self.permissioned_account_is_signer,
            members,
            Some(signer_seeds),
        )
    }
}
