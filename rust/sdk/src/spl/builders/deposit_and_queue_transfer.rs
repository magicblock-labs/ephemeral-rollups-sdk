use core::fmt;

use crate::{
    compat,
    consts::{EPHEMERAL_VAULT_ID, ESPL_TOKEN_PROGRAM_ID, MAGIC_PROGRAM_ID, TOKEN_PROGRAM_ID},
    spl::{find_transfer_group_receipt, EphemeralSplDiscriminator},
};

#[derive(Debug)]
pub enum DepositAndQueueTransferBuilderError {
    InvalidSplit(u32),
    RandomGroupIdUnavailable(getrandom::Error),
    InvalidDelayRange {
        min_delay_ms: u64,
        max_delay_ms: u64,
    },
}

impl fmt::Display for DepositAndQueueTransferBuilderError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidSplit(split) => write!(f, "split must be greater than zero, got {split}"),
            Self::RandomGroupIdUnavailable(err) => {
                write!(f, "failed to generate transfer group id: {err}")
            }
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
    pub queue: compat::Pubkey,
    pub vault: compat::Pubkey,
    pub mint: compat::Pubkey,
    pub source: compat::Pubkey,
    pub vault_ata: compat::Pubkey,
    pub destination: compat::Pubkey,
    pub owner: compat::Pubkey,
    pub reimbursement_token_info: Option<compat::Pubkey>,
    pub amount: u64,
    pub min_delay_ms: u64,
    pub max_delay_ms: u64,
    pub split: u32,
    pub client_ref_id: Option<u64>,
}

impl DepositAndQueueTransferBuilder {
    #[inline(always)]
    pub fn instruction(&self) -> Result<compat::Instruction, DepositAndQueueTransferBuilderError> {
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

        let group_id = random_transfer_group_id()?;
        let mut data = Vec::with_capacity(if self.client_ref_id.is_some() { 40 } else { 32 });
        data.push(EphemeralSplDiscriminator::DepositAndQueueTransfer as u8);
        data.extend_from_slice(&self.amount.to_le_bytes());
        data.extend_from_slice(&group_id.to_le_bytes()[..3]);
        data.extend_from_slice(&self.min_delay_ms.to_le_bytes());
        data.extend_from_slice(&self.max_delay_ms.to_le_bytes());
        data.extend_from_slice(&self.split.to_le_bytes());
        if let Some(client_ref_id) = self.client_ref_id {
            data.extend_from_slice(&client_ref_id.to_le_bytes());
        }
        let reimbursement_token_info = self.reimbursement_token_info.unwrap_or(self.source);
        let (group_receipt, _group_receipt_bump) =
            find_transfer_group_receipt(&self.queue, &self.owner, group_id);

        Ok(compat::Instruction {
            program_id: ESPL_TOKEN_PROGRAM_ID,
            accounts: vec![
                compat::AccountMeta::new(self.queue, false),
                compat::AccountMeta::new_readonly(self.vault, false),
                compat::AccountMeta::new_readonly(self.mint, false),
                compat::AccountMeta::new(self.source, false),
                compat::AccountMeta::new(self.vault_ata, false),
                compat::AccountMeta::new_readonly(self.destination, false),
                compat::AccountMeta::new_readonly(self.owner, true),
                compat::AccountMeta::new_readonly(TOKEN_PROGRAM_ID, false),
                compat::AccountMeta::new(reimbursement_token_info, false),
                compat::AccountMeta::new(group_receipt, false),
                compat::AccountMeta::new(EPHEMERAL_VAULT_ID, false),
                compat::AccountMeta::new_readonly(MAGIC_PROGRAM_ID, false),
            ],
            data,
        })
    }
}

fn random_transfer_group_id() -> Result<u32, DepositAndQueueTransferBuilderError> {
    let mut bytes = [0u8; 3];
    let mut group_id = 0;
    while group_id == 0 {
        getrandom::getrandom(&mut bytes)
            .map_err(DepositAndQueueTransferBuilderError::RandomGroupIdUnavailable)?;
        group_id = u32::from(bytes[0]) | (u32::from(bytes[1]) << 8) | (u32::from(bytes[2]) << 16);
    }
    Ok(group_id)
}
