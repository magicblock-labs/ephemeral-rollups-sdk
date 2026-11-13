use solana_program::program::invoke_signed;

use crate::access_control::structs::EphemeralMembersArgs;

pub const UPDATE_EPHEMERAL_PERMISSION_DISCRIMINATOR: u64 = 7;

#[derive(Debug)]
pub struct UpdateEphemeralPermission<'a> {
    pub permissioned_account: solana_program::pubkey::Pubkey,
    pub permission: solana_program::pubkey::Pubkey,
    pub payer: solana_program::pubkey::Pubkey,
    pub authority: solana_program::pubkey::Pubkey,
    pub vault: solana_program::pubkey::Pubkey,
    pub magic_program: solana_program::pubkey::Pubkey,
    pub permission_program: solana_program::pubkey::Pubkey,
    pub authority_is_signer: bool,
    pub args: &'a EphemeralMembersArgs,
}

impl<'a> UpdateEphemeralPermission<'a> {
    pub fn instruction(
        &self,
    ) -> Result<solana_program::instruction::Instruction, solana_program::program_error::ProgramError>
    {
        let accounts = vec![
            solana_program::instruction::AccountMeta::new(self.payer, true),
            solana_program::instruction::AccountMeta::new_readonly(
                self.authority,
                self.authority_is_signer,
            ),
            solana_program::instruction::AccountMeta::new_readonly(
                self.permissioned_account,
                !self.authority_is_signer,
            ),
            solana_program::instruction::AccountMeta::new(self.permission, false),
            solana_program::instruction::AccountMeta::new(self.vault, false),
            solana_program::instruction::AccountMeta::new_readonly(self.magic_program, false),
        ];
        let mut bytes = vec![0; EphemeralMembersArgs::required_bytes(self.args.members.len())];
        self.args
            .to_bytes(&mut bytes)
            .map_err(|_| solana_program::program_error::ProgramError::InvalidInstructionData)?;
        let data = [
            UPDATE_EPHEMERAL_PERMISSION_DISCRIMINATOR
                .to_le_bytes()
                .to_vec(),
            bytes,
        ]
        .concat();

        Ok(solana_program::instruction::Instruction {
            program_id: self.permission_program,
            accounts,
            data,
        })
    }
}

pub struct UpdateEphemeralPermissionCpi<'a> {
    pub permissioned_account: solana_program::account_info::AccountInfo<'a>,
    pub permission: solana_program::account_info::AccountInfo<'a>,
    pub payer: solana_program::account_info::AccountInfo<'a>,
    pub authority: solana_program::account_info::AccountInfo<'a>,
    pub vault: solana_program::account_info::AccountInfo<'a>,
    pub magic_program: solana_program::account_info::AccountInfo<'a>,
    pub permission_program: solana_program::account_info::AccountInfo<'a>,
    pub authority_is_signer: bool,
    pub args: EphemeralMembersArgs,
}

impl<'a> UpdateEphemeralPermissionCpi<'a> {
    pub fn invoke(self) -> solana_program::entrypoint_deprecated::ProgramResult {
        self.invoke_signed(&[])
    }

    pub fn invoke_signed(
        self,
        signers: &[&[&[u8]]],
    ) -> solana_program::entrypoint_deprecated::ProgramResult {
        let ix = UpdateEphemeralPermission {
            permissioned_account: *self.permissioned_account.key,
            permission: *self.permission.key,
            payer: *self.payer.key,
            authority: *self.authority.key,
            vault: *self.vault.key,
            magic_program: *self.magic_program.key,
            permission_program: *self.permission_program.key,
            authority_is_signer: self.authority_is_signer,
            args: &self.args,
        }
        .instruction()?;

        invoke_signed(
            &ix,
            &[
                self.payer,
                self.authority,
                self.permissioned_account,
                self.permission,
                self.vault,
                self.magic_program,
            ],
            signers,
        )
    }
}
