use solana_program::program::invoke_signed;

use crate::compat::{self, Compat, Modern};

pub const CLOSE_EPHEMERAL_PERMISSION_DISCRIMINATOR: u64 = 8;

/// Accounts.
#[derive(Debug)]
pub struct CloseEphemeralPermission {
    pub payer: compat::Pubkey,
    pub authority: compat::Pubkey,
    pub permissioned_account: compat::Pubkey,
    pub permission: compat::Pubkey,
    pub vault: compat::Pubkey,
    pub magic_program: compat::Pubkey,
    pub permission_program: compat::Pubkey,
    pub authority_is_signer: bool,
}

impl CloseEphemeralPermission {
    pub fn instruction(&self) -> compat::Instruction {
        let accounts = vec![
            compat::AccountMeta::new(self.payer, true),
            compat::AccountMeta::new_readonly(self.authority, self.authority_is_signer),
            compat::AccountMeta::new_readonly(self.permissioned_account, !self.authority_is_signer),
            compat::AccountMeta::new(self.permission, false),
            compat::AccountMeta::new(self.vault, false),
            compat::AccountMeta::new_readonly(self.magic_program, false),
        ];
        let data = CLOSE_EPHEMERAL_PERMISSION_DISCRIMINATOR
            .to_le_bytes()
            .to_vec();

        compat::Instruction {
            program_id: self.permission_program,
            accounts,
            data,
        }
    }
}

pub struct CloseEphemeralPermissionCpi<'a> {
    pub permissioned_account: compat::AccountInfo<'a>,
    pub permission: compat::AccountInfo<'a>,
    pub payer: compat::AccountInfo<'a>,
    pub authority: compat::AccountInfo<'a>,
    pub vault: compat::AccountInfo<'a>,
    pub magic_program: compat::AccountInfo<'a>,
    pub permission_program: compat::AccountInfo<'a>,
    pub authority_is_signer: bool,
}

impl<'a> CloseEphemeralPermissionCpi<'a> {
    pub fn invoke(self) -> compat::ProgramResult {
        self.invoke_signed(&[])
    }

    pub fn invoke_signed(self, signers: &[&[&[u8]]]) -> compat::ProgramResult {
        let ix = CloseEphemeralPermission {
            payer: *self.payer.key,
            authority: *self.authority.key,
            permissioned_account: *self.permissioned_account.key,
            permission: *self.permission.key,
            vault: *self.vault.key,
            magic_program: *self.magic_program.key,
            permission_program: *self.permission_program.key,
            authority_is_signer: self.authority_is_signer,
        }
        .instruction()
        .modern();

        invoke_signed(
            &ix,
            &[
                self.payer.modern(),
                self.authority.modern(),
                self.permissioned_account.modern(),
                self.permission.modern(),
                self.vault.modern(),
                self.magic_program.modern(),
            ],
            signers,
        )
        .compat()
    }
}
