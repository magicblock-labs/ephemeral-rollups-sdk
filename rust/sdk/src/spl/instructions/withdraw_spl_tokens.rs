use crate::{
    consts::{ESPL_TOKEN_PROGRAM_ID, TOKEN_PROGRAM_ID},
    solana_compat::solana::{AccountMeta, Instruction, Pubkey},
    spl::EphemeralSplDiscriminator,
};

/// Build a withdraw SPL tokens instruction.
pub fn withdraw_spl_tokens(
    payer: Pubkey,
    eata: Pubkey,
    vault: Pubkey,
    mint: Pubkey,
    vault_ata: Pubkey,
    user_ata: Pubkey,
    eata_bump: u8,
    amount: u64,
) -> Instruction {
    let mut data = Vec::with_capacity(10);
    data.push(EphemeralSplDiscriminator::WithdrawSplTokens as u8);
    data.extend_from_slice(amount.to_le_bytes().as_ref());
    data.push(eata_bump);
    Instruction {
        program_id: ESPL_TOKEN_PROGRAM_ID,
        accounts: vec![
            AccountMeta::new(eata, false),
            AccountMeta::new_readonly(vault, false),
            AccountMeta::new_readonly(mint, false),
            AccountMeta::new(vault_ata, false),
            AccountMeta::new(user_ata, false),
            AccountMeta::new_readonly(payer, true),
            AccountMeta::new_readonly(TOKEN_PROGRAM_ID, false),
        ],
        data,
    }
}
