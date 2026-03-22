use core::fmt;

use dlp_api::pda::{
    delegate_buffer_pda_from_delegated_account_and_owner_program,
    delegation_metadata_pda_from_delegated_account, delegation_record_pda_from_delegated_account,
};

use crate::{
    consts::{ASSOCIATED_TOKEN_PROGRAM_ID, ESPL_TOKEN_PROGRAM_ID, TOKEN_PROGRAM_ID},
    cpi::DELEGATION_PROGRAM_ID,
    solana_compat::solana::{system_program, AccountMeta, Instruction, Pubkey},
    spl::{
        find_rent_pda, find_shuttle_ata, find_shuttle_ephemeral_ata, find_shuttle_wallet_ata,
        find_transfer_queue, find_vault_ata, EphemeralSplDiscriminator, GlobalVault,
    },
};

const QUEUED_TRANSFER_FLAG_CREATE_IDEMPOTENT_ATA: u8 = 1 << 0;

#[derive(Debug)]
pub enum DepositAndDelegateShuttleEphemeralAtaWithMergeAndPrivateTransferBuilderError {
    MissingValidator,
    PrivateTransferPayloadTooLong(usize),
    Encryption(dlp_api::encryption::EncryptionError),
    #[cfg(not(feature = "encryption"))]
    EncryptionFeatureDisabled,
}

impl fmt::Display for DepositAndDelegateShuttleEphemeralAtaWithMergeAndPrivateTransferBuilderError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingValidator => {
                write!(f, "validator is required for encrypted private transfers")
            }
            Self::PrivateTransferPayloadTooLong(len) => {
                write!(
                    f,
                    "encrypted private transfer payload exceeds u8 length: {len}"
                )
            }
            Self::Encryption(err) => write!(f, "private transfer encryption failed: {err}"),
            #[cfg(not(feature = "encryption"))]
            Self::EncryptionFeatureDisabled => {
                write!(
                    f,
                    "enable the `encryption` feature for encrypted private transfers"
                )
            }
        }
    }
}

pub struct DepositAndDelegateShuttleEphemeralAtaWithMergeAndPrivateTransferBuilder {
    pub payer: Pubkey,
    pub owner: Pubkey,
    pub mint: Pubkey,
    pub source_ata: Pubkey,
    pub destination_owner: Pubkey,
    pub shuttle_id: u32,
    pub amount: u64,
    pub min_delay_ms: u64,
    pub max_delay_ms: u64,
    pub split: u32,
    pub validator: Option<Pubkey>,
}

impl DepositAndDelegateShuttleEphemeralAtaWithMergeAndPrivateTransferBuilder {
    #[inline(always)]
    pub fn instruction(
        &self,
    ) -> Result<
        Instruction,
        DepositAndDelegateShuttleEphemeralAtaWithMergeAndPrivateTransferBuilderError,
    > {
        let (rent_pda, _rent_bump) = find_rent_pda();
        let (shuttle_ephemeral_ata, _shuttle_bump) =
            find_shuttle_ephemeral_ata(&self.owner, &self.mint, self.shuttle_id);
        let (shuttle_ata, _shuttle_ata_bump) = find_shuttle_ata(&shuttle_ephemeral_ata, &self.mint);
        let shuttle_wallet_ata = find_shuttle_wallet_ata(&self.mint, &shuttle_ephemeral_ata);
        let delegation_buffer = delegate_buffer_pda_from_delegated_account_and_owner_program(
            &shuttle_ata,
            &ESPL_TOKEN_PROGRAM_ID,
        );
        let delegation_record = delegation_record_pda_from_delegated_account(&shuttle_ata);
        let delegation_metadata = delegation_metadata_pda_from_delegated_account(&shuttle_ata);
        let (vault, _vault_bump) = GlobalVault::find_pda(&self.mint);
        let vault_ata = find_vault_ata(&self.mint, &vault);
        let (queue, _queue_bump) = find_transfer_queue(&self.mint);
        let validator = self.require_validator()?;
        let encrypted_destination =
            encrypt_private_transfer_field(self.destination_owner.as_ref(), validator)?;
        let encrypted_suffix = encrypt_private_transfer_field(
            &pack_private_transfer_suffix(
                self.min_delay_ms,
                self.max_delay_ms,
                self.split,
                QUEUED_TRANSFER_FLAG_CREATE_IDEMPOTENT_ATA,
            ),
            validator,
        )?;

        let mut data = Vec::with_capacity(
            1 + 4 + 8 + 1 + 32 + 1 + encrypted_destination.len() + 1 + encrypted_suffix.len(),
        );
        data.push(
            EphemeralSplDiscriminator::DepositAndDelegateShuttleEphemeralAtaWithMergeAndPrivateTransfer
                as u8,
        );
        data.extend_from_slice(&self.shuttle_id.to_le_bytes());
        data.extend_from_slice(&self.amount.to_le_bytes());
        push_length_prefixed(&mut data, validator.as_ref())?;
        push_length_prefixed(&mut data, &encrypted_destination)?;
        push_length_prefixed(&mut data, &encrypted_suffix)?;

        Ok(Instruction {
            program_id: ESPL_TOKEN_PROGRAM_ID,
            accounts: vec![
                AccountMeta::new(self.payer, true),
                AccountMeta::new(rent_pda, false),
                AccountMeta::new(shuttle_ephemeral_ata, false),
                AccountMeta::new(shuttle_ata, false),
                AccountMeta::new(shuttle_wallet_ata, false),
                AccountMeta::new_readonly(self.owner, true),
                AccountMeta::new_readonly(ESPL_TOKEN_PROGRAM_ID, false),
                AccountMeta::new(delegation_buffer, false),
                AccountMeta::new(delegation_record, false),
                AccountMeta::new(delegation_metadata, false),
                AccountMeta::new_readonly(DELEGATION_PROGRAM_ID, false),
                AccountMeta::new_readonly(ASSOCIATED_TOKEN_PROGRAM_ID, false),
                AccountMeta::new_readonly(system_program::id(), false),
                AccountMeta::new_readonly(self.mint, false),
                AccountMeta::new_readonly(TOKEN_PROGRAM_ID, false),
                AccountMeta::new_readonly(vault, false),
                AccountMeta::new(self.source_ata, false),
                AccountMeta::new(vault_ata, false),
                AccountMeta::new(queue, false),
            ],
            data,
        })
    }

    #[inline(always)]
    fn require_validator(
        &self,
    ) -> Result<&Pubkey, DepositAndDelegateShuttleEphemeralAtaWithMergeAndPrivateTransferBuilderError>
    {
        self.validator.as_ref().ok_or(
            DepositAndDelegateShuttleEphemeralAtaWithMergeAndPrivateTransferBuilderError::MissingValidator,
        )
    }
}

#[inline(always)]
fn pack_private_transfer_suffix(
    min_delay_ms: u64,
    max_delay_ms: u64,
    split: u32,
    flags: u8,
) -> [u8; 21] {
    let mut suffix = [0u8; 21];
    suffix[..8].copy_from_slice(&min_delay_ms.to_le_bytes());
    suffix[8..16].copy_from_slice(&max_delay_ms.to_le_bytes());
    suffix[16..20].copy_from_slice(&split.to_le_bytes());
    suffix[20] = flags;
    suffix
}

#[inline(always)]
fn push_length_prefixed(
    data: &mut Vec<u8>,
    bytes: &[u8],
) -> Result<(), DepositAndDelegateShuttleEphemeralAtaWithMergeAndPrivateTransferBuilderError> {
    let len = u8::try_from(bytes.len()).map_err(|_| {
        DepositAndDelegateShuttleEphemeralAtaWithMergeAndPrivateTransferBuilderError::PrivateTransferPayloadTooLong(
            bytes.len(),
        )
    })?;
    data.push(len);
    data.extend_from_slice(bytes);
    Ok(())
}

#[cfg(feature = "encryption")]
#[inline(always)]
fn encrypt_private_transfer_field(
    plaintext: &[u8],
    validator: &Pubkey,
) -> Result<Vec<u8>, DepositAndDelegateShuttleEphemeralAtaWithMergeAndPrivateTransferBuilderError> {
    dlp_api::encryption::encrypt_ed25519_recipient(plaintext, validator.as_array()).map_err(
        DepositAndDelegateShuttleEphemeralAtaWithMergeAndPrivateTransferBuilderError::Encryption,
    )
}

#[cfg(not(feature = "encryption"))]
#[inline(always)]
fn encrypt_private_transfer_field(
    _plaintext: &[u8],
    _validator: &Pubkey,
) -> Result<Vec<u8>, DepositAndDelegateShuttleEphemeralAtaWithMergeAndPrivateTransferBuilderError> {
    Err(
        DepositAndDelegateShuttleEphemeralAtaWithMergeAndPrivateTransferBuilderError::EncryptionFeatureDisabled,
    )
}
