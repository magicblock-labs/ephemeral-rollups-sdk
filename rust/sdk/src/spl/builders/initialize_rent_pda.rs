use crate::{
    consts::ESPL_TOKEN_PROGRAM_ID,
    solana_compat::solana::{system_program, AccountMeta, Instruction, Pubkey},
    spl::{find_rent_pda, EphemeralSplDiscriminator},
};

pub struct InitializeRentPdaBuilder {
    pub payer: Pubkey,
}

impl InitializeRentPdaBuilder {
    #[inline(always)]
    pub fn instruction(&self) -> Instruction {
        let (rent_pda, _rent_bump) = find_rent_pda();

        Instruction {
            program_id: ESPL_TOKEN_PROGRAM_ID,
            accounts: vec![
                AccountMeta::new(self.payer, true),
                AccountMeta::new(rent_pda, false),
                AccountMeta::new_readonly(system_program::id(), false),
            ],
            data: vec![EphemeralSplDiscriminator::InitializeRentPda as u8],
        }
    }
}
