use pinocchio::{cpi::Seed, cpi::Signer, error::ProgramError, AccountView, Address, ProgramResult};

use crate::acl::{types::MembersArgs, utils::cpi_create_permission};
use crate::utils::make_seed_buf;

/// Create a new permission for a delegated account.
pub fn create_permission(
    accounts: &[&AccountView],
    permission_program: &Address,
    args: MembersArgs,
    signer_seeds: Option<Signer<'_, '_>>,
) -> ProgramResult {
    let [permissioned_account, permission, payer, system_program] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };

    if signer_seeds.is_none() && !permissioned_account.is_signer() {
        return Err(ProgramError::MissingRequiredSignature);
    }

    cpi_create_permission(
        permissioned_account,
        permission,
        payer,
        system_program,
        permission_program,
        args,
        signer_seeds,
    )
}

pub struct CreatePermissionCpiBuilder<'a> {
    permissioned_account: &'a AccountView,
    permission: &'a AccountView,
    payer: &'a AccountView,
    system_program: &'a AccountView,
    permission_program: &'a Address,
    members: Option<MembersArgs<'a>>,
    seeds: Option<&'a [&'a [u8]]>,
    bump: Option<u8>,
}

impl<'a> CreatePermissionCpiBuilder<'a> {
    pub fn new(
        permissioned_account: &'a AccountView,
        permission: &'a AccountView,
        payer: &'a AccountView,
        system_program: &'a AccountView,
        permission_program: &'a Address,
    ) -> Self {
        Self {
            permissioned_account,
            permission,
            payer,
            system_program,
            permission_program,
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
        let members = self.members.unwrap_or_else(MembersArgs::private);
        let seeds = self.seeds.ok_or(ProgramError::InvalidInstructionData)?;
        let bump = self.bump.ok_or(ProgramError::InvalidInstructionData)?;
        let mut seed_buf = make_seed_buf();
        if seeds.len() + 1 > seed_buf.len() {
            return Err(ProgramError::InvalidInstructionData);
        }
        let bump_slice = [bump];
        for (i, seed) in seeds.iter().enumerate() {
            seed_buf[i] = Seed::from(*seed);
        }
        seed_buf[seeds.len()] = Seed::from(&bump_slice);
        let signer_seeds = Signer::from(&seed_buf[..=seeds.len()]);

        create_permission(
            &[
                self.permissioned_account,
                self.permission,
                self.payer,
                self.system_program,
            ],
            self.permission_program,
            members,
            Some(signer_seeds),
        )
    }
}
