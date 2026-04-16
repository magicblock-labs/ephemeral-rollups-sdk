use dlp_api::pda::{
    delegate_buffer_pda_from_delegated_account_and_owner_program,
    delegation_metadata_pda_from_delegated_account, delegation_record_pda_from_delegated_account,
};

use crate::{
    consts::ESPL_TOKEN_PROGRAM_ID,
    cpi::DELEGATION_PROGRAM_ID,
    solana_compat::solana::{system_program, AccountMeta, Instruction, Pubkey},
    spl::{find_lamports_pda, find_rent_pda, EphemeralSplDiscriminator},
};

pub struct LamportsDelegatedTransferBuilder {
    pub payer: Pubkey,
    pub destination: Pubkey,
    pub amount: u64,
    pub salt: [u8; 32],
}

impl LamportsDelegatedTransferBuilder {
    #[inline(always)]
    pub fn instruction(&self) -> Instruction {
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

        Instruction {
            program_id: ESPL_TOKEN_PROGRAM_ID,
            accounts: vec![
                AccountMeta::new(self.payer, true),
                AccountMeta::new(rent_pda, false),
                AccountMeta::new(lamports_pda, false),
                AccountMeta::new_readonly(ESPL_TOKEN_PROGRAM_ID, false),
                AccountMeta::new(delegation_buffer, false),
                AccountMeta::new(delegation_record, false),
                AccountMeta::new(delegation_metadata, false),
                AccountMeta::new_readonly(DELEGATION_PROGRAM_ID, false),
                AccountMeta::new_readonly(system_program::id(), false),
                AccountMeta::new(self.destination, false),
                AccountMeta::new_readonly(destination_delegation_record, false),
            ],
            data,
        }
    }
}
