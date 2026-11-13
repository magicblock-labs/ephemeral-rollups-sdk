use solana_program::program::invoke_signed;

pub const CLOSE_EPHEMERAL_PERMISSION_DISCRIMINATOR: u64 = 8;

/// Accounts.
#[derive(Debug)]
pub struct CloseEphemeralPermission {
    pub payer: solana_program::pubkey::Pubkey,
    pub authority: solana_program::pubkey::Pubkey,
    pub permissioned_account: solana_program::pubkey::Pubkey,
    pub permission: solana_program::pubkey::Pubkey,
    pub vault: solana_program::pubkey::Pubkey,
    pub magic_program: solana_program::pubkey::Pubkey,
    pub permission_program: solana_program::pubkey::Pubkey,
    pub authority_is_signer: bool,
}

impl CloseEphemeralPermission {
    pub fn instruction(&self) -> solana_program::instruction::Instruction {
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
        let data = CLOSE_EPHEMERAL_PERMISSION_DISCRIMINATOR
            .to_le_bytes()
            .to_vec();

        solana_program::instruction::Instruction {
            program_id: self.permission_program,
            accounts,
            data,
        }
    }
}

pub struct CloseEphemeralPermissionCpi<'a> {
    pub permissioned_account: solana_program::account_info::AccountInfo<'a>,
    pub permission: solana_program::account_info::AccountInfo<'a>,
    pub payer: solana_program::account_info::AccountInfo<'a>,
    pub authority: solana_program::account_info::AccountInfo<'a>,
    pub vault: solana_program::account_info::AccountInfo<'a>,
    pub magic_program: solana_program::account_info::AccountInfo<'a>,
    pub permission_program: solana_program::account_info::AccountInfo<'a>,
    pub authority_is_signer: bool,
}

impl<'a> CloseEphemeralPermissionCpi<'a> {
    pub fn invoke(self) -> solana_program::entrypoint_deprecated::ProgramResult {
        self.invoke_signed(&[])
    }

    pub fn invoke_signed(
        self,
        signers: &[&[&[u8]]],
    ) -> solana_program::entrypoint_deprecated::ProgramResult {
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
        .instruction();

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
