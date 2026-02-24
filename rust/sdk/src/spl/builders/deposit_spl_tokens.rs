use spl_associated_token_account_interface::address::get_associated_token_address;

use crate::{
    consts::{ESPL_TOKEN_PROGRAM_ID, TOKEN_PROGRAM_ID},
    solana_compat::solana::{AccountMeta, Instruction, Pubkey},
    spl::{EphemeralAta, EphemeralSplDiscriminator, GlobalVault},
};

pub struct DepositSplTokensBuilder {
    pub authority: Pubkey,
    pub user: Pubkey,
    pub mint: Pubkey,
    pub amount: u64,
}

impl DepositSplTokensBuilder {
    #[inline(always)]
    pub fn instruction(&self) -> Instruction {
        let (eata, _eata_bump) = EphemeralAta::find_pda(&self.user, &self.mint);
        let (vault, _vault_bump) = GlobalVault::find_pda(&self.mint);
        let user_source_token_acc = get_associated_token_address(&self.user, &self.mint);
        let vault_token_acc = get_associated_token_address(&vault, &self.mint);

        let mut data = Vec::with_capacity(9);
        data.push(EphemeralSplDiscriminator::DepositSplTokens as u8);
        data.extend_from_slice(&self.amount.to_le_bytes());

        Instruction {
            program_id: ESPL_TOKEN_PROGRAM_ID,
            accounts: vec![
                AccountMeta::new(eata, false),
                AccountMeta::new_readonly(vault, false),
                AccountMeta::new_readonly(self.mint, false),
                AccountMeta::new(user_source_token_acc, false),
                AccountMeta::new(vault_token_acc, false),
                AccountMeta::new_readonly(self.authority, true),
                AccountMeta::new_readonly(TOKEN_PROGRAM_ID, false),
            ],
            data,
        }
    }
}
