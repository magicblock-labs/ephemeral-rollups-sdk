use crate::{
    compat,
    consts::{ESPL_TOKEN_PROGRAM_ID, TOKEN_PROGRAM_ID},
    spl::{find_shuttle_ephemeral_ata, find_shuttle_wallet_ata, EphemeralSplDiscriminator},
};

pub struct MergeShuttleIntoAtaBuilder {
    pub owner: compat::Pubkey,
    pub destination_ata: compat::Pubkey,
    pub mint: compat::Pubkey,
    pub shuttle_id: u32,
}

impl MergeShuttleIntoAtaBuilder {
    #[inline(always)]
    pub fn instruction(&self) -> compat::Instruction {
        let (shuttle_ephemeral_ata, _shuttle_bump) =
            find_shuttle_ephemeral_ata(&self.owner, &self.mint, self.shuttle_id);
        let shuttle_wallet_ata = find_shuttle_wallet_ata(&self.mint, &shuttle_ephemeral_ata);

        compat::Instruction {
            program_id: ESPL_TOKEN_PROGRAM_ID,
            accounts: vec![
                compat::AccountMeta::new_readonly(self.owner, true),
                compat::AccountMeta::new(self.destination_ata, false),
                compat::AccountMeta::new_readonly(shuttle_ephemeral_ata, false),
                compat::AccountMeta::new(shuttle_wallet_ata, false),
                compat::AccountMeta::new_readonly(self.mint, false),
                compat::AccountMeta::new_readonly(TOKEN_PROGRAM_ID, false),
            ],
            data: vec![EphemeralSplDiscriminator::MergeShuttleIntoAta as u8],
        }
    }
}
