use crate::{
    consts::{ASSOCIATED_TOKEN_PROGRAM_ID, ESPL_TOKEN_PROGRAM_ID, TOKEN_PROGRAM_ID},
    solana_compat::solana::{system_program, AccountMeta, Instruction, Pubkey},
    spl::{
        find_shuttle_ata, find_shuttle_ephemeral_ata, find_shuttle_wallet_ata,
        EphemeralSplDiscriminator,
    },
};

pub struct InitializeShuttleEphemeralAtaBuilder {
    pub payer: Pubkey,
    pub owner: Pubkey,
    pub mint: Pubkey,
    pub shuttle_id: u32,
}

impl InitializeShuttleEphemeralAtaBuilder {
    #[inline(always)]
    pub fn instruction(&self) -> Instruction {
        let (shuttle_ephemeral_ata, _shuttle_bump) =
            find_shuttle_ephemeral_ata(&self.owner, &self.mint, self.shuttle_id);
        let (shuttle_ata, _shuttle_ata_bump) = find_shuttle_ata(&shuttle_ephemeral_ata, &self.mint);
        let shuttle_wallet_ata = find_shuttle_wallet_ata(&self.mint, &shuttle_ephemeral_ata);

        let mut data = Vec::with_capacity(5);
        data.push(EphemeralSplDiscriminator::InitializeShuttleEphemeralAta as u8);
        data.extend_from_slice(&self.shuttle_id.to_le_bytes());

        Instruction {
            program_id: ESPL_TOKEN_PROGRAM_ID,
            accounts: vec![
                AccountMeta::new(self.payer, true),
                AccountMeta::new(shuttle_ephemeral_ata, false),
                AccountMeta::new(shuttle_ata, false),
                AccountMeta::new(shuttle_wallet_ata, false),
                AccountMeta::new_readonly(self.owner, false),
                AccountMeta::new_readonly(self.mint, false),
                AccountMeta::new_readonly(TOKEN_PROGRAM_ID, false),
                AccountMeta::new_readonly(ASSOCIATED_TOKEN_PROGRAM_ID, false),
                AccountMeta::new_readonly(system_program::id(), false),
            ],
            data,
        }
    }
}
