use core::fmt;

use crate::{
    consts::{ESPL_TOKEN_PROGRAM_ID, TOKEN_PROGRAM_ID},
    solana_compat::solana::{AccountMeta, Instruction, Pubkey},
    spl::EphemeralSplDiscriminator,
};

#[derive(Debug)]
pub enum DepositAndQueueTransferBuilderError {
    InvalidSplit(u32),
    InvalidDelayRange {
        min_delay_ms: u64,
        max_delay_ms: u64,
    },
}

impl fmt::Display for DepositAndQueueTransferBuilderError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidSplit(split) => write!(f, "split must be greater than zero, got {split}"),
            Self::InvalidDelayRange {
                min_delay_ms,
                max_delay_ms,
            } => write!(
                f,
                "max_delay_ms ({max_delay_ms}) must be greater than or equal to min_delay_ms ({min_delay_ms})"
            ),
        }
    }
}

pub struct DepositAndQueueTransferBuilder {
    pub queue: Pubkey,
    pub vault: Pubkey,
    pub mint: Pubkey,
    pub source: Pubkey,
    pub vault_ata: Pubkey,
    pub destination: Pubkey,
    pub owner: Pubkey,
    pub reimbursement_token_info: Option<Pubkey>,
    pub amount: u64,
    pub min_delay_ms: u64,
    pub max_delay_ms: u64,
    pub split: u32,
    pub client_ref_id: Option<u64>,
}

impl DepositAndQueueTransferBuilder {
    #[inline(always)]
    pub fn instruction(&self) -> Result<Instruction, DepositAndQueueTransferBuilderError> {
        if self.split == 0 {
            return Err(DepositAndQueueTransferBuilderError::InvalidSplit(
                self.split,
            ));
        }
        if self.max_delay_ms < self.min_delay_ms {
            return Err(DepositAndQueueTransferBuilderError::InvalidDelayRange {
                min_delay_ms: self.min_delay_ms,
                max_delay_ms: self.max_delay_ms,
            });
        }

        let mut data = Vec::with_capacity(if self.client_ref_id.is_some() { 37 } else { 29 });
        data.push(EphemeralSplDiscriminator::DepositAndQueueTransfer as u8);
        data.extend_from_slice(&self.amount.to_le_bytes());
        data.extend_from_slice(&self.min_delay_ms.to_le_bytes());
        data.extend_from_slice(&self.max_delay_ms.to_le_bytes());
        data.extend_from_slice(&self.split.to_le_bytes());
        if let Some(client_ref_id) = self.client_ref_id {
            data.extend_from_slice(&client_ref_id.to_le_bytes());
        }
        let reimbursement_token_info = self.reimbursement_token_info.unwrap_or(self.source);

        Ok(Instruction {
            program_id: ESPL_TOKEN_PROGRAM_ID,
            accounts: vec![
                AccountMeta::new(self.queue, false),
                AccountMeta::new_readonly(self.vault, false),
                AccountMeta::new_readonly(self.mint, false),
                AccountMeta::new(self.source, false),
                AccountMeta::new(self.vault_ata, false),
                AccountMeta::new_readonly(self.destination, false),
                AccountMeta::new_readonly(self.owner, true),
                AccountMeta::new_readonly(TOKEN_PROGRAM_ID, false),
                AccountMeta::new(reimbursement_token_info, false),
            ],
            data,
        })
    }
}
