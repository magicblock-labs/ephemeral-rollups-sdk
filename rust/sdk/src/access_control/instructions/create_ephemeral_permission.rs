use solana_program::program::invoke_signed;

use crate::access_control::structs::EphemeralMembersArgs;
use crate::compat::{
    AccountInfo, AccountMeta, AsModern, Compat, Instruction, Modern, ProgramError, ProgramResult,
    Pubkey,
};
use crate::modernize;

pub const CREATE_EPHEMERAL_PERMISSION_DISCRIMINATOR: u64 = 6;

#[derive(Debug)]
pub struct CreateEphemeralPermission<'a> {
    pub permissioned_account: Pubkey,
    pub permission: Pubkey,
    pub payer: Pubkey,
    pub vault: Pubkey,
    pub magic_program: Pubkey,
    pub permission_program: Pubkey,
    pub args: &'a EphemeralMembersArgs,
}

impl<'a> CreateEphemeralPermission<'a> {
    pub fn instruction(&self) -> Result<Instruction, ProgramError> {
        let accounts = vec![
            AccountMeta::new(self.payer, true),
            AccountMeta::new_readonly(self.permissioned_account, true),
            AccountMeta::new(self.permission, false),
            AccountMeta::new(self.vault, false),
            AccountMeta::new_readonly(self.magic_program, false),
        ];
        let mut bytes = vec![0; EphemeralMembersArgs::required_bytes(self.args.members.len())];
        self.args
            .to_bytes(&mut bytes)
            .map_err(|_| ProgramError::InvalidInstructionData)?;
        let data = [
            CREATE_EPHEMERAL_PERMISSION_DISCRIMINATOR
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

pub struct CreateEphemeralPermissionCpi<'a> {
    pub permissioned_account: AccountInfo<'a>,
    pub permission: AccountInfo<'a>,
    pub payer: AccountInfo<'a>,
    pub vault: AccountInfo<'a>,
    pub magic_program: AccountInfo<'a>,
    pub permission_program: AccountInfo<'a>,
    pub args: EphemeralMembersArgs,
}

impl<'a> CreateEphemeralPermissionCpi<'a> {
    pub fn invoke(self) -> ProgramResult {
        self.invoke_signed(&[])
    }

    pub fn invoke_signed(self, signers: &[&[&[u8]]]) -> ProgramResult {
        let CreateEphemeralPermissionCpi {
            payer,
            permissioned_account,
            permission,
            vault,
            magic_program,
            permission_program,
            args,
        } = self;
        modernize!(
            payer,
            permissioned_account,
            permission,
            vault,
            magic_program,
            permission_program,
        );

        let ix = CreateEphemeralPermission {
            permissioned_account: permissioned_account.key.compat(),
            permission: permission.key.compat(),
            payer: payer.key.compat(),
            vault: vault.key.compat(),
            magic_program: magic_program.key.compat(),
            permission_program: permission_program.key.compat(),
            args: &args,
        }
        .instruction()?
        .modern();

        invoke_signed(
            &ix,
            &[
                payer.clone(),
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
