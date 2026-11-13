use crate::{
    consts::ESPL_TOKEN_PROGRAM_ID,
    spl::{find_rent_pda, EphemeralSplDiscriminator},
};

pub struct InitializeRentPdaBuilder {
    pub payer: solana_program::pubkey::Pubkey,
}

impl InitializeRentPdaBuilder {
    #[inline(always)]
    pub fn instruction(&self) -> solana_program::instruction::Instruction {
        let (rent_pda, _rent_bump) = find_rent_pda();

        solana_program::instruction::Instruction {
            program_id: ESPL_TOKEN_PROGRAM_ID,
            accounts: vec![
                solana_program::instruction::AccountMeta::new(self.payer, true),
                solana_program::instruction::AccountMeta::new(rent_pda, false),
                solana_program::instruction::AccountMeta::new_readonly(
                    solana_program::pubkey::Pubkey::new_from_array(
                        solana_system_interface::program::ID.to_bytes(),
                    ),
                    false,
                ),
            ],
            data: vec![EphemeralSplDiscriminator::InitializeRentPda as u8],
        }
    }
}
