use crate::{
    consts::{ESPL_TOKEN_PROGRAM_ID, PERMISSION_PROGRAM_ID},
    cpi::DELEGATION_PROGRAM_ID,
    solana_compat::solana::{AccountMeta, Instruction, Pubkey},
    spl::EphemeralSplDiscriminator,
};

/// Delegate an ephemeral ATA permission.
#[allow(clippy::too_many_arguments)]
pub fn delegate_ephemeral_ata_permission(
    payer: Pubkey,
    eata: Pubkey,
    permission: Pubkey,
    system_program: Pubkey,
    delegation_buffer: Pubkey,
    delegation_record: Pubkey,
    delegation_metadata: Pubkey,
    validator: Pubkey,
    eata_bump: u8,
) -> Instruction {
    Instruction {
        program_id: ESPL_TOKEN_PROGRAM_ID,
        accounts: vec![
            AccountMeta::new(payer, true),
            AccountMeta::new(eata, false),
            AccountMeta::new_readonly(PERMISSION_PROGRAM_ID, false),
            AccountMeta::new(permission, false),
            AccountMeta::new_readonly(system_program, false),
            AccountMeta::new(delegation_buffer, false),
            AccountMeta::new(delegation_record, false),
            AccountMeta::new(delegation_metadata, false),
            AccountMeta::new_readonly(DELEGATION_PROGRAM_ID, false),
            AccountMeta::new_readonly(validator, false),
        ],
        data: vec![
            EphemeralSplDiscriminator::DelegateEphemeralAtaPermission as u8,
            eata_bump,
        ],
    }
}
