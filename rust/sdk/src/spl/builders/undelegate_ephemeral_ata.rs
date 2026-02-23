use spl_associated_token_account_interface::address::get_associated_token_address;

use crate::{
    consts::{ESPL_TOKEN_PROGRAM_ID, MAGIC_CONTEXT_ID, MAGIC_PROGRAM_ID},
    solana_compat::solana::{AccountMeta, Instruction, Pubkey},
    spl::{cpi::EphemeralSplDiscriminator, EphemeralAta},
};

pub struct UndelegateEphemeralAtaBuilder {
    pub payer: Pubkey,
    pub user: Pubkey,
    pub mint: Pubkey,
}

impl UndelegateEphemeralAtaBuilder {
    #[inline(always)]
    pub fn instruction(&self) -> Instruction {
        let (eata, _eata_bump) = EphemeralAta::find_pda(&self.user, &self.mint);
        let user_ata = get_associated_token_address(&self.user, &self.mint);

        Instruction {
            program_id: ESPL_TOKEN_PROGRAM_ID,
            accounts: vec![
                AccountMeta::new_readonly(self.payer, true),
                AccountMeta::new(user_ata, false),
                AccountMeta::new_readonly(eata, false),
                AccountMeta::new(MAGIC_CONTEXT_ID, false),
                AccountMeta::new_readonly(MAGIC_PROGRAM_ID, false),
            ],
            data: vec![EphemeralSplDiscriminator::UndelegateEphemeralAta as u8],
        }
    }
}
