use crate::{
    compat,
    consts::ESPL_TOKEN_PROGRAM_ID,
    spl::{EphemeralAta, EphemeralSplDiscriminator},
};

pub struct InitializeEphemeralAtaBuilder {
    pub payer: compat::Pubkey,
    pub user: compat::Pubkey,
    pub mint: compat::Pubkey,
}

impl InitializeEphemeralAtaBuilder {
    #[inline(always)]
    pub fn instruction(&self) -> compat::Instruction {
        let (eata, _eata_bump) = EphemeralAta::find_pda(&self.user, &self.mint);
        compat::Instruction {
            program_id: ESPL_TOKEN_PROGRAM_ID,
            accounts: vec![
                compat::AccountMeta::new(eata, false),
                compat::AccountMeta::new(self.payer, true),
                compat::AccountMeta::new_readonly(self.user, false),
                compat::AccountMeta::new_readonly(self.mint, false),
                compat::AccountMeta::new_readonly(
                    compat::Pubkey::new_from_array(solana_system_interface::program::ID.to_bytes()),
                    false,
                ),
            ],
            data: vec![EphemeralSplDiscriminator::InitializeEphemeralAta as u8],
        }
    }
}
