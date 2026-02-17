use crate::{
    cpi::DELEGATION_PROGRAM_ID,
    solana_compat::solana::{system_program, AccountMeta, Instruction, Pubkey},
    spl::EphemeralSplDiscriminator,
};

use crate::consts::ESPL_TOKEN_PROGRAM_ID;

/// Delegate an ephemeral ATA.
pub fn delegate_ephemeral_ata(
    payer: Pubkey,
    eata: Pubkey,
    delegation_buffer: Pubkey,
    delegation_record: Pubkey,
    delegation_metadata: Pubkey,
    eata_bump: u8,
    validator: Option<Pubkey>,
) -> Instruction {
    let mut data = Vec::with_capacity(34);
    data.push(EphemeralSplDiscriminator::DelegateEphemeralAta as u8);
    data.push(eata_bump);
    if let Some(validator) = validator {
        data.extend_from_slice(validator.to_bytes().as_ref());
    }
    Instruction {
        program_id: ESPL_TOKEN_PROGRAM_ID,
        accounts: vec![
            AccountMeta::new(payer, true),
            AccountMeta::new(eata, false),
            AccountMeta::new_readonly(ESPL_TOKEN_PROGRAM_ID, false),
            AccountMeta::new(delegation_buffer, false),
            AccountMeta::new(delegation_record, false),
            AccountMeta::new(delegation_metadata, false),
            AccountMeta::new_readonly(DELEGATION_PROGRAM_ID, false),
            AccountMeta::new_readonly(system_program::id(), false),
        ],
        data,
    }
}
