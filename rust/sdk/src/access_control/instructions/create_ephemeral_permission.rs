use crate::access_control::structs::EphemeralMembersArgs;
use crate::solana_compat::solana::{
    invoke_signed, AccountInfo, AccountMeta, Instruction, ProgramError, ProgramResult, Pubkey,
};

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
    pub fn invoke(&self) -> ProgramResult {
        self.invoke_signed(&[])
    }

    pub fn invoke_signed(&self, signers: &[&[&[u8]]]) -> ProgramResult {
        let ix = CreateEphemeralPermission {
            permissioned_account: *self.permissioned_account.key,
            permission: *self.permission.key,
            payer: *self.payer.key,
            vault: *self.vault.key,
            magic_program: *self.magic_program.key,
            permission_program: *self.permission_program.key,
            args: &self.args,
        }
        .instruction()?;

        invoke_signed(
            &ix,
            &[
                self.payer.clone(),
                self.permissioned_account.clone(),
                self.permission.clone(),
                self.vault.clone(),
                self.magic_program.clone(),
            ],
            signers,
        )
    }
}
