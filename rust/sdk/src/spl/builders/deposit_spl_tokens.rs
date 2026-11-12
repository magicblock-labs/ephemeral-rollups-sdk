use crate::{
    consts::{ESPL_TOKEN_PROGRAM_ID, TOKEN_PROGRAM_ID},
    spl::{
        find_associated_token_address_with_bump, find_vault_ata, EphemeralAta,
        EphemeralSplDiscriminator, GlobalVault,
    },
};

pub struct DepositSplTokensBuilder {
    pub authority: solana_program::pubkey::Pubkey,
    pub user: solana_program::pubkey::Pubkey,
    pub mint: solana_program::pubkey::Pubkey,
    pub amount: u64,
}

impl DepositSplTokensBuilder {
    #[inline(always)]
    pub fn instruction(&self) -> solana_program::instruction::Instruction {
        let (eata, _eata_bump) = EphemeralAta::find_pda(&self.user, &self.mint);
        let (vault, _vault_bump) = GlobalVault::find_pda(&self.mint);
        let user_source_token_acc =
            find_associated_token_address_with_bump(&self.user, &self.mint, &TOKEN_PROGRAM_ID).0;
        let vault_token_acc = find_vault_ata(&self.mint, &vault);

        let mut data = Vec::with_capacity(9);
        data.push(EphemeralSplDiscriminator::DepositSplTokens as u8);
        data.extend_from_slice(&self.amount.to_le_bytes());

        solana_program::instruction::Instruction {
            program_id: ESPL_TOKEN_PROGRAM_ID,
            accounts: vec![
                solana_program::instruction::AccountMeta::new(eata, false),
                solana_program::instruction::AccountMeta::new_readonly(vault, false),
                solana_program::instruction::AccountMeta::new_readonly(self.mint, false),
                solana_program::instruction::AccountMeta::new(user_source_token_acc, false),
                solana_program::instruction::AccountMeta::new(vault_token_acc, false),
                solana_program::instruction::AccountMeta::new_readonly(self.authority, true),
                solana_program::instruction::AccountMeta::new_readonly(TOKEN_PROGRAM_ID, false),
            ],
            data,
        }
    }
}
