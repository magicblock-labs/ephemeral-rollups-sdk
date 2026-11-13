use crate::{
    consts::{ESPL_TOKEN_PROGRAM_ID, TOKEN_PROGRAM_ID},
    spl::{find_shuttle_ephemeral_ata, find_shuttle_wallet_ata, EphemeralSplDiscriminator},
};

pub struct MergeShuttleIntoAtaBuilder {
    pub owner: solana_program::pubkey::Pubkey,
    pub destination_ata: solana_program::pubkey::Pubkey,
    pub mint: solana_program::pubkey::Pubkey,
    pub shuttle_id: u32,
}

impl MergeShuttleIntoAtaBuilder {
    #[inline(always)]
    pub fn instruction(&self) -> solana_program::instruction::Instruction {
        let (shuttle_ephemeral_ata, _shuttle_bump) =
            find_shuttle_ephemeral_ata(&self.owner, &self.mint, self.shuttle_id);
        let shuttle_wallet_ata = find_shuttle_wallet_ata(&self.mint, &shuttle_ephemeral_ata);

        solana_program::instruction::Instruction {
            program_id: ESPL_TOKEN_PROGRAM_ID,
            accounts: vec![
                solana_program::instruction::AccountMeta::new_readonly(self.owner, true),
                solana_program::instruction::AccountMeta::new(self.destination_ata, false),
                solana_program::instruction::AccountMeta::new_readonly(
                    shuttle_ephemeral_ata,
                    false,
                ),
                solana_program::instruction::AccountMeta::new(shuttle_wallet_ata, false),
                solana_program::instruction::AccountMeta::new_readonly(self.mint, false),
                solana_program::instruction::AccountMeta::new_readonly(TOKEN_PROGRAM_ID, false),
            ],
            data: vec![EphemeralSplDiscriminator::MergeShuttleIntoAta as u8],
        }
    }
}
