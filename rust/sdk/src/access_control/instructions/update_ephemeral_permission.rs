use crate::access_control::structs::EphemeralMembersArgs;
use crate::solana_compat::solana::{
    invoke_signed, AccountInfo, AccountMeta, Instruction, ProgramResult, Pubkey,
};

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
    pub fn instruction(&self) -> Instruction {
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
            .expect("Failed to serialize members args");
        let data = [
            UPDATE_EPHEMERAL_PERMISSION_DISCRIMINATOR
                .to_le_bytes()
                .to_vec(),
            bytes,
        ]
        .concat();

        Instruction {
            program_id: self.permission_program,
            accounts,
            data,
        }
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
    pub fn invoke(&self) -> ProgramResult {
        self.invoke_signed(&[])
    }

    pub fn invoke_signed(&self, signers: &[&[&[u8]]]) -> ProgramResult {
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
        .instruction();

        invoke_signed(
            &ix,
            &[
                self.payer.clone(),
                self.authority.clone(),
                self.permissioned_account.clone(),
                self.permission.clone(),
                self.vault.clone(),
                self.magic_program.clone(),
            ],
            signers,
        )
    }
}
