use crate::{
    consts::{ESPL_TOKEN_PROGRAM_ID, MAGIC_CONTEXT_ID, MAGIC_PROGRAM_ID, PERMISSION_PROGRAM_ID},
    solana_compat::solana::{AccountMeta, Instruction, Pubkey},
    spl::EphemeralSplDiscriminator,
};

/// Undelegate an ephemeral ATA permission.
pub fn undelegate_ephemeral_ata_permission(
    payer: Pubkey,
    eata: Pubkey,
    permission: Pubkey,
) -> Instruction {
    Instruction {
        program_id: ESPL_TOKEN_PROGRAM_ID,
        accounts: vec![
            AccountMeta::new_readonly(payer, true),
            AccountMeta::new_readonly(eata, false),
            AccountMeta::new(permission, false),
            AccountMeta::new_readonly(PERMISSION_PROGRAM_ID, false),
            AccountMeta::new_readonly(MAGIC_PROGRAM_ID, false),
            AccountMeta::new(MAGIC_CONTEXT_ID, false),
        ],
        data: vec![EphemeralSplDiscriminator::UndelegateEphemeralAtaPermission as u8],
    }
}
