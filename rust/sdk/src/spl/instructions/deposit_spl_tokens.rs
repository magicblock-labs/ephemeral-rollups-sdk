use crate::{
    consts::{ESPL_TOKEN_PROGRAM_ID, TOKEN_PROGRAM_ID},
    solana_compat::solana::{AccountMeta, Instruction, Pubkey},
    spl::EphemeralSplDiscriminator,
};

/// Deposit SPL tokens into an ephemeral ATA.
pub fn deposit_spl_tokens(
    authority: Pubkey,
    eata: Pubkey,
    vault: Pubkey,
    mint: Pubkey,
    user_source_token_acc: Pubkey,
    vault_token_acc: Pubkey,
    amount: u64,
) -> Instruction {
    let mut data = Vec::with_capacity(9);
    data.push(EphemeralSplDiscriminator::DepositSplTokens as u8);
    data.extend_from_slice(amount.to_le_bytes().as_ref());
    Instruction {
        program_id: ESPL_TOKEN_PROGRAM_ID,
        accounts: vec![
            AccountMeta::new(eata, false),
            AccountMeta::new_readonly(vault, false),
            AccountMeta::new_readonly(mint, false),
            AccountMeta::new(user_source_token_acc, false),
            AccountMeta::new(vault_token_acc, false),
            AccountMeta::new_readonly(authority, true),
            AccountMeta::new_readonly(TOKEN_PROGRAM_ID, false),
        ],
        data,
    }
}
