use crate::{
    compat,
    consts::ESPL_TOKEN_PROGRAM_ID,
    spl::{find_rent_pda, EphemeralSplDiscriminator},
};

pub struct InitializeRentPdaBuilder {
    pub payer: compat::Pubkey,
}

impl InitializeRentPdaBuilder {
    #[inline(always)]
    pub fn instruction(&self) -> compat::Instruction {
        let (rent_pda, _rent_bump) = find_rent_pda();

        compat::Instruction {
            program_id: ESPL_TOKEN_PROGRAM_ID,
            accounts: vec![
                compat::AccountMeta::new(self.payer, true),
                compat::AccountMeta::new(rent_pda, false),
                compat::AccountMeta::new_readonly(
                    compat::Pubkey::new_from_array(solana_system_interface::program::ID.to_bytes()),
                    false,
                ),
            ],
            data: vec![EphemeralSplDiscriminator::InitializeRentPda as u8],
        }
    }
}
