use spl_associated_token_account_interface::address::get_associated_token_address;

use crate::{
    consts::{ASSOCIATED_TOKEN_PROGRAM_ID, ESPL_TOKEN_PROGRAM_ID, TOKEN_PROGRAM_ID},
    solana_compat::solana::{system_program, AccountMeta, Instruction, Pubkey},
    spl::{EphemeralAta, EphemeralSplDiscriminator, GlobalVault},
};

pub struct InitializeGlobalVaultBuilder {
    pub payer: Pubkey,
    pub mint: Pubkey,
}

impl InitializeGlobalVaultBuilder {
    #[inline(always)]
    pub fn instruction(&self) -> Instruction {
        let (vault, _vault_bump) = GlobalVault::find_pda(&self.mint);
        let (vault_ephemeral_ata, _vault_eata_bump) = EphemeralAta::find_pda(&vault, &self.mint);
        let vault_ata = get_associated_token_address(&vault, &self.mint);
        Instruction {
            program_id: ESPL_TOKEN_PROGRAM_ID,
            accounts: vec![
                AccountMeta::new(vault, false),
                AccountMeta::new(self.payer, true),
                AccountMeta::new_readonly(self.mint, false),
                AccountMeta::new(vault_ephemeral_ata, false),
                AccountMeta::new(vault_ata, false),
                AccountMeta::new_readonly(TOKEN_PROGRAM_ID, false),
                AccountMeta::new_readonly(ASSOCIATED_TOKEN_PROGRAM_ID, false),
                AccountMeta::new_readonly(system_program::id(), false),
            ],
            data: vec![EphemeralSplDiscriminator::InitializeGlobalVault as u8],
        }
    }
}
