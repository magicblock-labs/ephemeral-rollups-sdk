use crate::consts::PERMISSION_PROGRAM_ID;
use crate::solana_compat::solana::{
    invoke_signed, AccountInfo, AccountMeta, Instruction, ProgramResult, Pubkey,
};

pub const CLOSE_EPHEMERAL_PERMISSION_DISCRIMINATOR: u64 = 8;

/// Accounts.
#[derive(Debug)]
pub struct CloseEphemeralPermission {
    pub payer: Pubkey,
    pub authority: Pubkey,
    pub permissioned_account: Pubkey,
    pub permission: Pubkey,
    pub vault: Pubkey,
    pub magic_program: Pubkey,
    pub authority_is_signer: bool,
}

impl CloseEphemeralPermission {
    pub fn instruction(&self) -> Instruction {
        let accounts = vec![
            AccountMeta::new(self.payer, true),
            AccountMeta::new_readonly(self.authority, self.authority_is_signer),
            AccountMeta::new_readonly(self.permissioned_account, !self.authority_is_signer),
            AccountMeta::new(self.permission, false),
            AccountMeta::new(self.vault, false),
            AccountMeta::new_readonly(self.magic_program, false),
        ];
        let data = CLOSE_EPHEMERAL_PERMISSION_DISCRIMINATOR
            .to_le_bytes()
            .to_vec();

        Instruction {
            program_id: PERMISSION_PROGRAM_ID,
            accounts,
            data,
        }
    }
}

pub struct CloseEphemeralPermissionCpi<'a> {
    pub permissioned_account: AccountInfo<'a>,
    pub permission: AccountInfo<'a>,
    pub payer: AccountInfo<'a>,
    pub authority: AccountInfo<'a>,
    pub vault: AccountInfo<'a>,
    pub magic_program: AccountInfo<'a>,
    pub permission_program: AccountInfo<'a>,
    pub authority_is_signer: bool,
}

impl<'a> CloseEphemeralPermissionCpi<'a> {
    pub fn invoke(&self) -> ProgramResult {
        self.invoke_signed(&[])
    }

    pub fn invoke_signed(&self, signers: &[&[&[u8]]]) -> ProgramResult {
        let ix = CloseEphemeralPermission {
            payer: *self.payer.key,
            authority: *self.authority.key,
            permissioned_account: *self.permissioned_account.key,
            permission: *self.permission.key,
            vault: *self.vault.key,
            magic_program: *self.magic_program.key,
            authority_is_signer: self.authority_is_signer,
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
