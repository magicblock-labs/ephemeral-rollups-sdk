use pinocchio::cpi::invoke_signed;
use pinocchio::instruction::{InstructionAccount, InstructionView};
use pinocchio::{cpi::Signer, AccountView, ProgramResult};

use crate::acl::{EphemeralMembersArgs, CREATE_EPHEMERAL_PERMISSION_DISCRIMINATOR};

pub struct CreateEphemeralPermission<'a> {
    pub permissioned_account: &'a AccountView,
    pub permission: &'a AccountView,
    pub payer: &'a AccountView,
    pub vault: &'a AccountView,
    pub magic_program: &'a AccountView,
    pub permission_program: &'a AccountView,
    pub args: EphemeralMembersArgs<'a>,
}

impl<'a> CreateEphemeralPermission<'a> {
    /// N is the size of the data buffer, depending on the number of members in the args.
    pub fn invoke<const N: usize>(&self) -> ProgramResult {
        self.invoke_signed::<N>(&[])
    }

    /// N is the size of the data buffer, depending on the number of members in the args.
    pub fn invoke_signed<const N: usize>(&self, signers: &[Signer<'_, '_>]) -> ProgramResult {
        let mut data = [0_u8; N];
        data[0..8].copy_from_slice(&CREATE_EPHEMERAL_PERMISSION_DISCRIMINATOR.to_le_bytes());
        let len = self.args.to_bytes(&mut data[8..])?;
        invoke_signed(
            &InstructionView {
                program_id: self.permission_program.address(),
                accounts: &[
                    InstructionAccount::readonly_signer(self.permissioned_account.address()),
                    InstructionAccount::writable(self.permission.address()),
                    InstructionAccount::writable_signer(self.payer.address()),
                    InstructionAccount::writable(self.vault.address()),
                    InstructionAccount::readonly(self.magic_program.address()),
                ],
                data: &data[..8 + len],
            },
            &[
                self.permissioned_account,
                self.permission,
                self.payer,
                self.vault,
                self.magic_program,
            ],
            signers,
        )
    }
}
