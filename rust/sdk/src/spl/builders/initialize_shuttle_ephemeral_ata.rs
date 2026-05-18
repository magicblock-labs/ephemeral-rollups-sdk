use crate::{
    compat,
    consts::{ASSOCIATED_TOKEN_PROGRAM_ID, ESPL_TOKEN_PROGRAM_ID, TOKEN_PROGRAM_ID},
    spl::{
        find_shuttle_ata, find_shuttle_ephemeral_ata, find_shuttle_wallet_ata,
        EphemeralSplDiscriminator,
    },
};

pub struct InitializeShuttleEphemeralAtaBuilder {
    pub payer: compat::Pubkey,
    pub owner: compat::Pubkey,
    pub mint: compat::Pubkey,
    pub shuttle_id: u32,
}

impl InitializeShuttleEphemeralAtaBuilder {
    #[inline(always)]
    pub fn instruction(&self) -> compat::Instruction {
        let (shuttle_ephemeral_ata, _shuttle_bump) =
            find_shuttle_ephemeral_ata(&self.owner, &self.mint, self.shuttle_id);
        let (shuttle_ata, _shuttle_ata_bump) = find_shuttle_ata(&shuttle_ephemeral_ata, &self.mint);
        let shuttle_wallet_ata = find_shuttle_wallet_ata(&self.mint, &shuttle_ephemeral_ata);

        let mut data = Vec::with_capacity(5);
        data.push(EphemeralSplDiscriminator::InitializeShuttleEphemeralAta as u8);
        data.extend_from_slice(&self.shuttle_id.to_le_bytes());

        compat::Instruction {
            program_id: ESPL_TOKEN_PROGRAM_ID,
            accounts: vec![
                compat::AccountMeta::new(self.payer, true),
                compat::AccountMeta::new(shuttle_ephemeral_ata, false),
                compat::AccountMeta::new(shuttle_ata, false),
                compat::AccountMeta::new(shuttle_wallet_ata, false),
                compat::AccountMeta::new_readonly(self.owner, false),
                compat::AccountMeta::new_readonly(self.mint, false),
                compat::AccountMeta::new_readonly(TOKEN_PROGRAM_ID, false),
                compat::AccountMeta::new_readonly(ASSOCIATED_TOKEN_PROGRAM_ID, false),
                compat::AccountMeta::new_readonly(
                    compat::Pubkey::new_from_array(solana_system_interface::program::ID.to_bytes()),
                    false,
                ),
            ],
            data,
        }
    }
}
