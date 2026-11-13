use crate::{
    consts::{ASSOCIATED_TOKEN_PROGRAM_ID, ESPL_TOKEN_PROGRAM_ID, TOKEN_PROGRAM_ID},
    spl::{find_vault_ata, EphemeralAta, EphemeralSplDiscriminator, GlobalVault},
};

pub struct InitializeGlobalVaultBuilder {
    pub payer: solana_program::pubkey::Pubkey,
    pub mint: solana_program::pubkey::Pubkey,
}

impl InitializeGlobalVaultBuilder {
    #[inline(always)]
    pub fn instruction(&self) -> solana_program::instruction::Instruction {
        let (vault, _vault_bump) = GlobalVault::find_pda(&self.mint);
        let (vault_ephemeral_ata, _vault_eata_bump) = EphemeralAta::find_pda(&vault, &self.mint);
        let vault_ata = find_vault_ata(&self.mint, &vault);
        solana_program::instruction::Instruction {
            program_id: ESPL_TOKEN_PROGRAM_ID,
            accounts: vec![
                solana_program::instruction::AccountMeta::new(vault, false),
                solana_program::instruction::AccountMeta::new(self.payer, true),
                solana_program::instruction::AccountMeta::new_readonly(self.mint, false),
                solana_program::instruction::AccountMeta::new(vault_ephemeral_ata, false),
                solana_program::instruction::AccountMeta::new(vault_ata, false),
                solana_program::instruction::AccountMeta::new_readonly(TOKEN_PROGRAM_ID, false),
                solana_program::instruction::AccountMeta::new_readonly(
                    ASSOCIATED_TOKEN_PROGRAM_ID,
                    false,
                ),
                solana_program::instruction::AccountMeta::new_readonly(
                    solana_program::pubkey::Pubkey::new_from_array(
                        solana_system_interface::program::ID.to_bytes(),
                    ),
                    false,
                ),
            ],
            data: vec![EphemeralSplDiscriminator::InitializeGlobalVault as u8],
        }
    }
}
