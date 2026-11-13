use crate::{
    consts::{ESPL_TOKEN_PROGRAM_ID, MAGIC_CONTEXT_ID, MAGIC_PROGRAM_ID, TOKEN_PROGRAM_ID},
    spl::{
        find_shuttle_ata, find_shuttle_ephemeral_ata, find_shuttle_wallet_ata,
        EphemeralSplDiscriminator,
    },
};

pub struct UndelegateAndCloseShuttleEphemeralAtaBuilder {
    pub payer: solana_program::pubkey::Pubkey,
    pub rent_reimbursement: solana_program::pubkey::Pubkey,
    pub owner: solana_program::pubkey::Pubkey,
    pub mint: solana_program::pubkey::Pubkey,
    pub destination_ata: solana_program::pubkey::Pubkey,
    pub shuttle_id: u32,
    pub escrow_index: Option<u8>,
}

impl UndelegateAndCloseShuttleEphemeralAtaBuilder {
    #[inline(always)]
    pub fn instruction(&self) -> solana_program::instruction::Instruction {
        let (shuttle_ephemeral_ata, _shuttle_bump) =
            find_shuttle_ephemeral_ata(&self.owner, &self.mint, self.shuttle_id);
        let (shuttle_ata, _shuttle_ata_bump) = find_shuttle_ata(&shuttle_ephemeral_ata, &self.mint);
        let shuttle_wallet_ata = find_shuttle_wallet_ata(&self.mint, &shuttle_ephemeral_ata);

        let mut data = Vec::with_capacity(2);
        data.push(EphemeralSplDiscriminator::UndelegateAndCloseShuttleEphemeralAta as u8);
        if let Some(escrow_index) = self.escrow_index {
            data.push(escrow_index);
        }

        solana_program::instruction::Instruction {
            program_id: ESPL_TOKEN_PROGRAM_ID,
            accounts: vec![
                solana_program::instruction::AccountMeta::new(self.payer, true),
                solana_program::instruction::AccountMeta::new(self.rent_reimbursement, false),
                solana_program::instruction::AccountMeta::new_readonly(
                    shuttle_ephemeral_ata,
                    false,
                ),
                solana_program::instruction::AccountMeta::new_readonly(shuttle_ata, false),
                solana_program::instruction::AccountMeta::new(shuttle_wallet_ata, false),
                solana_program::instruction::AccountMeta::new(self.destination_ata, false),
                solana_program::instruction::AccountMeta::new_readonly(TOKEN_PROGRAM_ID, false),
                solana_program::instruction::AccountMeta::new(MAGIC_CONTEXT_ID, false),
                solana_program::instruction::AccountMeta::new_readonly(MAGIC_PROGRAM_ID, false),
            ],
            data,
        }
    }
}
