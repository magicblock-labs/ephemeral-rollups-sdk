use crate::{
    consts::{ESPL_TOKEN_PROGRAM_ID, MAGIC_CONTEXT_ID, MAGIC_PROGRAM_ID},
    solana_compat::solana::{AccountMeta, Instruction, Pubkey},
    spl::EphemeralSplDiscriminator,
};

/// Build an undelegate ephemeral ATA instruction.
pub fn undelegate_ephemeral_ata(payer: Pubkey, user_ata: Pubkey, eata: Pubkey) -> Instruction {
    Instruction {
        program_id: ESPL_TOKEN_PROGRAM_ID,
        accounts: vec![
            AccountMeta::new_readonly(payer, true),
            AccountMeta::new(user_ata, false),
            AccountMeta::new_readonly(eata, false),
            AccountMeta::new(MAGIC_CONTEXT_ID, false),
            AccountMeta::new_readonly(MAGIC_PROGRAM_ID, false),
        ],
        data: vec![EphemeralSplDiscriminator::UndelegateEphemeralAta as u8],
    }
}
