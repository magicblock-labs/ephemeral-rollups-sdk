use crate::{
    consts::ESPL_TOKEN_PROGRAM_ID,
    solana_compat::solana::{system_program, AccountMeta, Instruction, Pubkey},
    spl::EphemeralSplDiscriminator,
};

/// Create an initialize ephemeral ATA instruction.
pub fn initialize_ephemeral_ata(
    payer: Pubkey,
    eata: Pubkey,
    user: Pubkey,
    mint: Pubkey,
    eata_bump: u8,
) -> Instruction {
    Instruction {
        program_id: ESPL_TOKEN_PROGRAM_ID,
        accounts: vec![
            AccountMeta::new(eata, false),
            AccountMeta::new(payer, false),
            AccountMeta::new_readonly(user, false),
            AccountMeta::new_readonly(mint, false),
            AccountMeta::new_readonly(system_program::id(), false),
        ],
        data: vec![
            EphemeralSplDiscriminator::InitializeEphemeralAta as u8,
            eata_bump,
        ],
    }
}
