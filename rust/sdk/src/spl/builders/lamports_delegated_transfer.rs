use crate::spl::compat_pda::{
    delegate_buffer_pda_from_delegated_account_and_owner_program,
    delegation_metadata_pda_from_delegated_account, delegation_record_pda_from_delegated_account,
};

use crate::{
    compat,
    consts::ESPL_TOKEN_PROGRAM_ID,
    cpi::DELEGATION_PROGRAM_ID,
    spl::{find_lamports_pda, find_rent_pda, EphemeralSplDiscriminator},
};

pub struct LamportsDelegatedTransferBuilder {
    pub payer: compat::Pubkey,
    pub destination: compat::Pubkey,
    pub amount: u64,
    pub salt: [u8; 32],
}

impl LamportsDelegatedTransferBuilder {
    #[inline(always)]
    pub fn instruction(&self) -> compat::Instruction {
        let (rent_pda, _rent_bump) = find_rent_pda();
        let (lamports_pda, _lamports_bump) =
            find_lamports_pda(&self.payer, &self.destination, &self.salt);
        let delegation_buffer = delegate_buffer_pda_from_delegated_account_and_owner_program(
            &lamports_pda,
            &ESPL_TOKEN_PROGRAM_ID,
        );
        let delegation_record = delegation_record_pda_from_delegated_account(&lamports_pda);
        let delegation_metadata = delegation_metadata_pda_from_delegated_account(&lamports_pda);
        let destination_delegation_record =
            delegation_record_pda_from_delegated_account(&self.destination);

        let mut data = Vec::with_capacity(41);
        data.push(EphemeralSplDiscriminator::LamportsDelegatedTransfer as u8);
        data.extend_from_slice(&self.amount.to_le_bytes());
        data.extend_from_slice(&self.salt);

        compat::Instruction {
            program_id: ESPL_TOKEN_PROGRAM_ID,
            accounts: vec![
                compat::AccountMeta::new(self.payer, true),
                compat::AccountMeta::new(rent_pda, false),
                compat::AccountMeta::new(lamports_pda, false),
                compat::AccountMeta::new_readonly(ESPL_TOKEN_PROGRAM_ID, false),
                compat::AccountMeta::new(delegation_buffer, false),
                compat::AccountMeta::new(delegation_record, false),
                compat::AccountMeta::new(delegation_metadata, false),
                compat::AccountMeta::new_readonly(DELEGATION_PROGRAM_ID, false),
                compat::AccountMeta::new_readonly(
                    compat::Pubkey::new_from_array(solana_system_interface::program::ID.to_bytes()),
                    false,
                ),
                compat::AccountMeta::new(self.destination, false),
                compat::AccountMeta::new_readonly(destination_delegation_record, false),
            ],
            data,
        }
    }
}
