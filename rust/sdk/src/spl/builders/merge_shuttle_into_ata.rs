use crate::{
    consts::{ESPL_TOKEN_PROGRAM_ID, TOKEN_PROGRAM_ID},
    solana_compat::solana::{AccountMeta, Instruction, Pubkey},
    spl::{find_shuttle_ephemeral_ata, find_shuttle_wallet_ata, EphemeralSplDiscriminator},
};

pub struct MergeShuttleIntoAtaBuilder {
    pub owner: Pubkey,
    pub destination_ata: Pubkey,
    pub mint: Pubkey,
    pub shuttle_id: u32,
}

impl MergeShuttleIntoAtaBuilder {
    #[inline(always)]
    pub fn instruction(&self) -> Instruction {
        let (shuttle_ephemeral_ata, _shuttle_bump) =
            find_shuttle_ephemeral_ata(&self.owner, &self.mint, self.shuttle_id);
        let shuttle_wallet_ata = find_shuttle_wallet_ata(&self.mint, &shuttle_ephemeral_ata);

        Instruction {
            program_id: ESPL_TOKEN_PROGRAM_ID,
            accounts: vec![
                AccountMeta::new_readonly(self.owner, true),
                AccountMeta::new(self.destination_ata, false),
                AccountMeta::new_readonly(shuttle_ephemeral_ata, false),
                AccountMeta::new(shuttle_wallet_ata, false),
                AccountMeta::new_readonly(self.mint, false),
                AccountMeta::new_readonly(TOKEN_PROGRAM_ID, false),
            ],
            data: vec![EphemeralSplDiscriminator::MergeShuttleIntoAta as u8],
        }
    }
}
