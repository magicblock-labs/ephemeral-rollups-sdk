use pinocchio::cpi::invoke_signed;
use pinocchio::instruction::{InstructionAccount, InstructionView};
use pinocchio::{cpi::Signer, AccountView, ProgramResult};

use crate::acl::{EphemeralMembersArgs, CLOSE_EPHEMERAL_PERMISSION_DISCRIMINATOR};

pub struct CloseEphemeralPermission<'a> {
    pub permissioned_account: &'a AccountView,
    pub permission: &'a AccountView,
    pub payer: &'a AccountView,
    pub authority: &'a AccountView,
    pub vault: &'a AccountView,
    pub magic_program: &'a AccountView,
    pub permission_program: &'a AccountView,
    pub authority_is_signer: bool,
    pub args: EphemeralMembersArgs<'a>,
}

impl<'a> CloseEphemeralPermission<'a> {
    pub fn invoke(&self) -> ProgramResult {
        self.invoke_signed(&[])
    }

    pub fn invoke_signed(&self, signers: &[Signer<'_, '_>]) -> ProgramResult {
        invoke_signed(
            &InstructionView {
                program_id: self.permission_program.address(),
                accounts: &[
                    InstructionAccount::new(
                        self.permissioned_account.address(),
                        false,
                        !self.authority_is_signer,
                    ),
                    InstructionAccount::writable(self.permission.address()),
                    InstructionAccount::new(
                        self.authority.address(),
                        false,
                        self.authority_is_signer,
                    ),
                    InstructionAccount::writable_signer(self.payer.address()),
                    InstructionAccount::writable(self.vault.address()),
                    InstructionAccount::readonly(self.magic_program.address()),
                ],
                data: &CLOSE_EPHEMERAL_PERMISSION_DISCRIMINATOR.to_le_bytes(),
            },
            &[
                self.permissioned_account,
                self.permission,
                self.authority,
                self.payer,
                self.vault,
                self.magic_program,
            ],
            signers,
        )
    }
}
