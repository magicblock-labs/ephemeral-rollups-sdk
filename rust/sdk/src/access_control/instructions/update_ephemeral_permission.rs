use solana_program::program::invoke_signed;

use crate::access_control::structs::EphemeralMembersArgs;
use crate::compat::{
    AccountInfo, AccountMeta, AsModern, Compat, Instruction, Modern, ProgramError, ProgramResult,
    Pubkey,
};
use crate::modernize;

pub const UPDATE_EPHEMERAL_PERMISSION_DISCRIMINATOR: u64 = 7;

#[derive(Debug)]
pub struct UpdateEphemeralPermission<'a> {
    pub permissioned_account: Pubkey,
    pub permission: Pubkey,
    pub payer: Pubkey,
    pub authority: Pubkey,
    pub vault: Pubkey,
    pub magic_program: Pubkey,
    pub permission_program: Pubkey,
    pub authority_is_signer: bool,
    pub args: &'a EphemeralMembersArgs,
}

impl<'a> UpdateEphemeralPermission<'a> {
    pub fn instruction(&self) -> Result<Instruction, ProgramError> {
        let accounts = vec![
            AccountMeta::new(self.payer, true),
            AccountMeta::new_readonly(self.authority, self.authority_is_signer),
            AccountMeta::new_readonly(self.permissioned_account, !self.authority_is_signer),
            AccountMeta::new(self.permission, false),
            AccountMeta::new(self.vault, false),
            AccountMeta::new_readonly(self.magic_program, false),
        ];
        let mut bytes = vec![0; EphemeralMembersArgs::required_bytes(self.args.members.len())];
        self.args
            .to_bytes(&mut bytes)
            .map_err(|_| ProgramError::InvalidInstructionData)?;
        let data = [
            UPDATE_EPHEMERAL_PERMISSION_DISCRIMINATOR
                .to_le_bytes()
                .to_vec(),
            bytes,
        ]
        .concat();

        Ok(Instruction {
            program_id: self.permission_program,
            accounts,
            data,
        })
    }
}

pub struct UpdateEphemeralPermissionCpi<'a> {
    pub permissioned_account: AccountInfo<'a>,
    pub permission: AccountInfo<'a>,
    pub payer: AccountInfo<'a>,
    pub authority: AccountInfo<'a>,
    pub vault: AccountInfo<'a>,
    pub magic_program: AccountInfo<'a>,
    pub permission_program: AccountInfo<'a>,
    pub authority_is_signer: bool,
    pub args: EphemeralMembersArgs,
}

impl<'a> UpdateEphemeralPermissionCpi<'a> {
    pub fn invoke(self) -> ProgramResult {
        self.invoke_signed(&[])
    }

    pub fn invoke_signed(self, signers: &[&[&[u8]]]) -> ProgramResult {
        let UpdateEphemeralPermissionCpi {
            permissioned_account,
            permission,
            payer,
            authority,
            vault,
            magic_program,
            permission_program,
            authority_is_signer,
            args,
        } = self;
        modernize!(
            payer,
            permissioned_account,
            permission,
            authority,
            vault,
            magic_program,
            permission_program,
        );

        let ix = UpdateEphemeralPermission {
            permissioned_account: permissioned_account.key.compat(),
            permission: permission.key.compat(),
            payer: payer.key.compat(),
            authority: authority.key.compat(),
            vault: vault.key.compat(),
            magic_program: magic_program.key.compat(),
            permission_program: permission_program.key.compat(),
            authority_is_signer,
            args: &args,
        }
        .instruction()?
        .modern();

        invoke_signed(
            &ix,
            &[
                payer.clone(),
                authority.clone(),
                permissioned_account.clone(),
                permission.clone(),
                vault.clone(),
                magic_program.clone(),
            ],
            signers,
        )
        .compat()
    }
}
