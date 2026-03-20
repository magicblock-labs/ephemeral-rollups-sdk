use spl_associated_token_account_interface::address::get_associated_token_address;

use crate::{
    consts::{ESPL_TOKEN_PROGRAM_ID, TOKEN_PROGRAM_ID},
    solana_compat::solana::{AccountMeta, Instruction, Pubkey},
    spl::{EphemeralAta, EphemeralSplDiscriminator, GlobalVault},
};

pub struct WithdrawSplTokensBuilder {
    pub payer: Pubkey,
    pub user: Pubkey,
    pub mint: Pubkey,
    pub amount: u64,
}

impl WithdrawSplTokensBuilder {
    #[inline(always)]
    pub fn instruction(&self) -> Instruction {
        let (eata, _eata_bump) = EphemeralAta::find_pda(&self.user, &self.mint);
        let (vault, vault_bump) = GlobalVault::find_pda(&self.mint);
        let vault_ata = get_associated_token_address(&vault, &self.mint);
        let user_ata = get_associated_token_address(&self.user, &self.mint);

        let mut data = Vec::with_capacity(10);
        data.push(EphemeralSplDiscriminator::WithdrawSplTokens as u8);
        data.extend_from_slice(&self.amount.to_le_bytes());
        data.push(vault_bump);

        Instruction {
            program_id: ESPL_TOKEN_PROGRAM_ID,
            accounts: vec![
                AccountMeta::new(eata, false),
                AccountMeta::new_readonly(vault, false),
                AccountMeta::new_readonly(self.mint, false),
                AccountMeta::new(vault_ata, false),
                AccountMeta::new(user_ata, false),
                AccountMeta::new_readonly(self.payer, true),
                AccountMeta::new_readonly(TOKEN_PROGRAM_ID, false),
            ],
            data,
        }
    }
}
