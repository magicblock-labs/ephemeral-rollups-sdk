use crate::{
    compat,
    consts::{ESPL_TOKEN_PROGRAM_ID, MAGIC_CONTEXT_ID, MAGIC_PROGRAM_ID, TOKEN_PROGRAM_ID},
    spl::{
        find_shuttle_ata, find_shuttle_ephemeral_ata, find_shuttle_wallet_ata,
        EphemeralSplDiscriminator,
    },
};

pub struct UndelegateAndCloseShuttleEphemeralAtaBuilder {
    pub payer: compat::Pubkey,
    pub rent_reimbursement: compat::Pubkey,
    pub owner: compat::Pubkey,
    pub mint: compat::Pubkey,
    pub destination_ata: compat::Pubkey,
    pub shuttle_id: u32,
    pub escrow_index: Option<u8>,
}

impl UndelegateAndCloseShuttleEphemeralAtaBuilder {
    #[inline(always)]
    pub fn instruction(&self) -> compat::Instruction {
        let (shuttle_ephemeral_ata, _shuttle_bump) =
            find_shuttle_ephemeral_ata(&self.owner, &self.mint, self.shuttle_id);
        let (shuttle_ata, _shuttle_ata_bump) = find_shuttle_ata(&shuttle_ephemeral_ata, &self.mint);
        let shuttle_wallet_ata = find_shuttle_wallet_ata(&self.mint, &shuttle_ephemeral_ata);

        let mut data = Vec::with_capacity(2);
        data.push(EphemeralSplDiscriminator::UndelegateAndCloseShuttleEphemeralAta as u8);
        if let Some(escrow_index) = self.escrow_index {
            data.push(escrow_index);
        }

        compat::Instruction {
            program_id: ESPL_TOKEN_PROGRAM_ID,
            accounts: vec![
                compat::AccountMeta::new(self.payer, true),
                compat::AccountMeta::new(self.rent_reimbursement, false),
                compat::AccountMeta::new_readonly(shuttle_ephemeral_ata, false),
                compat::AccountMeta::new_readonly(shuttle_ata, false),
                compat::AccountMeta::new(shuttle_wallet_ata, false),
                compat::AccountMeta::new(self.destination_ata, false),
                compat::AccountMeta::new_readonly(TOKEN_PROGRAM_ID, false),
                compat::AccountMeta::new(MAGIC_CONTEXT_ID, false),
                compat::AccountMeta::new_readonly(MAGIC_PROGRAM_ID, false),
            ],
            data,
        }
    }
}
