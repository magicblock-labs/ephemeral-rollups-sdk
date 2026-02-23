use crate::{
    consts::ESPL_TOKEN_PROGRAM_ID,
    solana_compat::solana::{system_program, AccountMeta, Instruction, Pubkey},
    spl::{cpi::EphemeralSplDiscriminator, GlobalVault},
};

pub struct InitializeGlobalVaultBuilder {
    pub payer: Pubkey,
    pub mint: Pubkey,
}

impl InitializeGlobalVaultBuilder {
    #[inline(always)]
    pub fn instruction(&self) -> Instruction {
        let (vault, vault_bump) = GlobalVault::find_pda(&self.mint);
        Instruction {
            program_id: ESPL_TOKEN_PROGRAM_ID,
            accounts: vec![
                AccountMeta::new(vault, false),
                AccountMeta::new(self.payer, true),
                AccountMeta::new_readonly(self.mint, false),
                AccountMeta::new_readonly(system_program::id(), false),
            ],
            data: vec![
                EphemeralSplDiscriminator::InitializeGlobalVault as u8,
                vault_bump,
            ],
        }
    }
}
