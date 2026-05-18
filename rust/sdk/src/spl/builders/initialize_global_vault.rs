use crate::{
    compat,
    consts::{ASSOCIATED_TOKEN_PROGRAM_ID, ESPL_TOKEN_PROGRAM_ID, TOKEN_PROGRAM_ID},
    spl::{find_vault_ata, EphemeralAta, EphemeralSplDiscriminator, GlobalVault},
};

pub struct InitializeGlobalVaultBuilder {
    pub payer: compat::Pubkey,
    pub mint: compat::Pubkey,
}

impl InitializeGlobalVaultBuilder {
    #[inline(always)]
    pub fn instruction(&self) -> compat::Instruction {
        let (vault, _vault_bump) = GlobalVault::find_pda(&self.mint);
        let (vault_ephemeral_ata, _vault_eata_bump) = EphemeralAta::find_pda(&vault, &self.mint);
        let vault_ata = find_vault_ata(&self.mint, &vault);
        compat::Instruction {
            program_id: ESPL_TOKEN_PROGRAM_ID,
            accounts: vec![
                compat::AccountMeta::new(vault, false),
                compat::AccountMeta::new(self.payer, true),
                compat::AccountMeta::new_readonly(self.mint, false),
                compat::AccountMeta::new(vault_ephemeral_ata, false),
                compat::AccountMeta::new(vault_ata, false),
                compat::AccountMeta::new_readonly(TOKEN_PROGRAM_ID, false),
                compat::AccountMeta::new_readonly(ASSOCIATED_TOKEN_PROGRAM_ID, false),
                compat::AccountMeta::new_readonly(
                    compat::Pubkey::new_from_array(solana_system_interface::program::ID.to_bytes()),
                    false,
                ),
            ],
            data: vec![EphemeralSplDiscriminator::InitializeGlobalVault as u8],
        }
    }
}
