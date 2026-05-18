use core::fmt;

use crate::{
    compat,
    consts::{ESPL_TOKEN_PROGRAM_ID, HYDRA_PROGRAM_ID},
    cpi::DELEGATION_PROGRAM_ID,
    spl::{
        find_hydra_crank_pda, find_rent_pda, find_shuttle_ata, find_shuttle_ephemeral_ata,
        find_stash_ata, find_stash_pda, find_transfer_queue,
        types::find_associated_token_address_with_bump, EphemeralSplDiscriminator, GlobalVault,
    },
};

const BUFFER_SEED: &[u8] = b"buffer";
const DELEGATION_RECORD_SEED: &[u8] = b"delegation";
const DELEGATION_METADATA_SEED: &[u8] = b"delegation-metadata";

#[derive(Debug)]
pub enum SchedulePrivateTransferBuilderError {
    InvalidSplit(u32),
    InvalidDelayRange {
        min_delay_ms: u64,
        max_delay_ms: u64,
    },
    PrivateTransferPayloadTooLong(usize),
    Encryption(dlp_api::encryption::EncryptionError),
}

impl fmt::Display for SchedulePrivateTransferBuilderError {
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
            Self::PrivateTransferPayloadTooLong(len) => {
                write!(f, "encrypted private transfer payload exceeds u8 length: {len}")
            }
            Self::Encryption(err) => write!(f, "private transfer encryption failed: {err}"),
        }
    }
}

pub struct SchedulePrivateTransferBuilder {
    pub user: compat::Pubkey,
    pub mint: compat::Pubkey,
    pub shuttle_id: u32,
    pub destination_owner: compat::Pubkey,
    pub min_delay_ms: u64,
    pub max_delay_ms: u64,
    pub split: u32,
    pub validator: compat::Pubkey,
    pub token_program: compat::Pubkey,
    pub client_ref_id: Option<u64>,
}

impl SchedulePrivateTransferBuilder {
    #[inline(always)]
    pub fn instruction(&self) -> Result<compat::Instruction, SchedulePrivateTransferBuilderError> {
        if self.split == 0 {
            return Err(SchedulePrivateTransferBuilderError::InvalidSplit(
                self.split,
            ));
        }
        if self.max_delay_ms < self.min_delay_ms {
            return Err(SchedulePrivateTransferBuilderError::InvalidDelayRange {
                min_delay_ms: self.min_delay_ms,
                max_delay_ms: self.max_delay_ms,
            });
        }

        let (stash_pda, stash_bump) = find_stash_pda(&self.user, &self.mint);
        let (_stash_ata, stash_ata_bump) =
            find_stash_ata(&self.user, &self.mint, &self.token_program);
        let (rent_pda, _rent_bump) = find_rent_pda();
        let (shuttle_ephemeral_ata, shuttle_bump) =
            find_shuttle_ephemeral_ata(&stash_pda, &self.mint, self.shuttle_id);
        let (shuttle_ata, shuttle_ata_bump) = find_shuttle_ata(&shuttle_ephemeral_ata, &self.mint);
        let (_shuttle_wallet_ata, shuttle_wallet_ata_bump) =
            find_associated_token_address_with_bump(
                &shuttle_ephemeral_ata,
                &self.mint,
                &self.token_program,
            );
        let (_buffer, buffer_bump) = compat::Pubkey::find_program_address(
            &[BUFFER_SEED, shuttle_ata.as_ref()],
            &ESPL_TOKEN_PROGRAM_ID,
        );
        let (_delegation_record, delegation_record_bump) = compat::Pubkey::find_program_address(
            &[DELEGATION_RECORD_SEED, shuttle_ata.as_ref()],
            &DELEGATION_PROGRAM_ID,
        );
        let (_delegation_metadata, delegation_metadata_bump) = compat::Pubkey::find_program_address(
            &[DELEGATION_METADATA_SEED, shuttle_ata.as_ref()],
            &DELEGATION_PROGRAM_ID,
        );
        let (vault, global_vault_bump) = GlobalVault::find_pda(&self.mint);
        let (_vault_ata, vault_token_bump) =
            find_associated_token_address_with_bump(&vault, &self.mint, &self.token_program);
        let (_queue, queue_bump) = find_transfer_queue(&self.mint, &self.validator);
        let (hydra_crank_pda, _hydra_crank_bump) =
            find_hydra_crank_pda(&stash_pda, self.shuttle_id);

        let encrypted_destination =
            encrypt_private_transfer_field(self.destination_owner.as_ref(), &self.validator)?;
        let encrypted_suffix = encrypt_private_transfer_field(
            &pack_private_transfer_suffix(
                self.min_delay_ms,
                self.max_delay_ms,
                self.split,
                self.client_ref_id,
            ),
            &self.validator,
        )?;

        let mut data = Vec::with_capacity(
            1 + 4
                + 1
                + 32
                + 10
                + 1
                + 32
                + 1
                + encrypted_destination.len()
                + 1
                + encrypted_suffix.len(),
        );
        data.push(EphemeralSplDiscriminator::SchedulePrivateTransfer as u8);
        data.extend_from_slice(&self.shuttle_id.to_le_bytes());
        data.push(stash_bump);
        data.extend_from_slice(self.mint.as_ref());
        data.push(shuttle_bump);
        data.push(shuttle_ata_bump);
        data.push(shuttle_wallet_ata_bump);
        data.push(buffer_bump);
        data.push(delegation_record_bump);
        data.push(delegation_metadata_bump);
        data.push(global_vault_bump);
        data.push(vault_token_bump);
        data.push(stash_ata_bump);
        data.push(queue_bump);
        push_length_prefixed(&mut data, self.validator.as_ref())?;
        push_length_prefixed(&mut data, &encrypted_destination)?;
        push_length_prefixed(&mut data, &encrypted_suffix)?;

        Ok(compat::Instruction {
            program_id: ESPL_TOKEN_PROGRAM_ID,
            accounts: vec![
                compat::AccountMeta::new(self.user, true),
                compat::AccountMeta::new(stash_pda, false),
                compat::AccountMeta::new(rent_pda, false),
                compat::AccountMeta::new(hydra_crank_pda, false),
                compat::AccountMeta::new_readonly(HYDRA_PROGRAM_ID, false),
                compat::AccountMeta::new_readonly(
                    compat::Pubkey::new_from_array(solana_system_interface::program::ID.to_bytes()),
                    false,
                ),
                compat::AccountMeta::new_readonly(self.token_program, false),
            ],
            data,
        })
    }
}

#[inline(always)]
fn pack_private_transfer_suffix(
    min_delay_ms: u64,
    max_delay_ms: u64,
    split: u32,
    client_ref_id: Option<u64>,
) -> Vec<u8> {
    let mut suffix = Vec::with_capacity(if client_ref_id.is_some() { 28 } else { 20 });
    suffix.extend_from_slice(&min_delay_ms.to_le_bytes());
    suffix.extend_from_slice(&max_delay_ms.to_le_bytes());
    suffix.extend_from_slice(&split.to_le_bytes());
    if let Some(client_ref_id) = client_ref_id {
        suffix.extend_from_slice(&client_ref_id.to_le_bytes());
    }
    suffix
}

#[inline(always)]
fn push_length_prefixed(
    data: &mut Vec<u8>,
    bytes: &[u8],
) -> Result<(), SchedulePrivateTransferBuilderError> {
    let len = u8::try_from(bytes.len()).map_err(|_| {
        SchedulePrivateTransferBuilderError::PrivateTransferPayloadTooLong(bytes.len())
    })?;
    data.push(len);
    data.extend_from_slice(bytes);
    Ok(())
}

#[inline(always)]
fn encrypt_private_transfer_field(
    plaintext: &[u8],
    validator: &compat::Pubkey,
) -> Result<Vec<u8>, SchedulePrivateTransferBuilderError> {
    dlp_api::encryption::encrypt_ed25519_recipient(plaintext, validator.as_array())
        .map_err(SchedulePrivateTransferBuilderError::Encryption)
}
