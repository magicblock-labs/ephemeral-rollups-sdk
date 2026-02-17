use crate::{
    consts::ESPL_TOKEN_PROGRAM_ID,
    solana_compat::solana::{system_program, AccountMeta, Instruction, Pubkey},
    spl::EphemeralSplDiscriminator,
};

/// Create an initialize global vault instruction.
pub fn initialize_global_vault(
    payer: Pubkey,
    vault: Pubkey,
    mint: Pubkey,
    vault_bump: u8,
) -> Instruction {
    Instruction {
        program_id: ESPL_TOKEN_PROGRAM_ID,
        accounts: vec![
            AccountMeta::new(vault, false),
            AccountMeta::new(payer, false),
            AccountMeta::new_readonly(mint, false),
            AccountMeta::new_readonly(system_program::id(), false),
        ],
        data: vec![
            EphemeralSplDiscriminator::InitializeGlobalVault as u8,
            vault_bump,
        ],
    }
}
