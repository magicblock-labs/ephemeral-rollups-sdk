use crate::consts::{
    ASSOCIATED_TOKEN_PROGRAM_ID, ESPL_TOKEN_PROGRAM_ID, HYDRA_PROGRAM_ID, TOKEN_PROGRAM_ID,
};

/// Internal representation of a token account data.
#[repr(C)]
pub struct EphemeralAta {
    /// The owner of the eata
    pub owner: solana_program::pubkey::Pubkey,
    /// The mint associated with this account
    pub mint: solana_program::pubkey::Pubkey,
    /// The amount of tokens this account holds.
    pub amount: u64,
}

impl EphemeralAta {
    pub const LEN: usize = 32 + 32 + 8;

    pub fn find_pda(
        user: &solana_program::pubkey::Pubkey,
        mint: &solana_program::pubkey::Pubkey,
    ) -> (solana_program::pubkey::Pubkey, u8) {
        solana_program::pubkey::Pubkey::find_program_address(
            &[user.as_ref(), mint.as_ref()],
            &ESPL_TOKEN_PROGRAM_ID,
        )
    }

    pub fn try_from_bytes(
        bytes: &[u8],
    ) -> Result<Self, solana_program::program_error::ProgramError> {
        if bytes.len() != Self::LEN {
            return Err(solana_program::program_error::ProgramError::InvalidAccountData);
        }
        Ok(Self {
            owner: solana_program::pubkey::Pubkey::new_from_array(
                bytes[0..32]
                    .try_into()
                    .map_err(|_| solana_program::program_error::ProgramError::InvalidAccountData)?,
            ),
            mint: solana_program::pubkey::Pubkey::new_from_array(
                bytes[32..64]
                    .try_into()
                    .map_err(|_| solana_program::program_error::ProgramError::InvalidAccountData)?,
            ),
            amount: u64::from_le_bytes(
                bytes[64..72]
                    .try_into()
                    .map_err(|_| solana_program::program_error::ProgramError::InvalidAccountData)?,
            ),
        })
    }
}

/// Internal representation of a global vault for a specific mint.
#[repr(C)]
pub struct GlobalVault {
    /// The mint associated with this vault
    pub mint: solana_program::pubkey::Pubkey,
}

impl GlobalVault {
    pub const LEN: usize = 32;

    pub fn find_pda(mint: &solana_program::pubkey::Pubkey) -> (solana_program::pubkey::Pubkey, u8) {
        solana_program::pubkey::Pubkey::find_program_address(
            &[mint.as_ref()],
            &ESPL_TOKEN_PROGRAM_ID,
        )
    }

    pub fn try_from_bytes(
        bytes: &[u8],
    ) -> Result<Self, solana_program::program_error::ProgramError> {
        if bytes.len() != Self::LEN {
            return Err(solana_program::program_error::ProgramError::InvalidAccountData);
        }
        Ok(Self {
            mint: solana_program::pubkey::Pubkey::new_from_array(
                bytes[0..32]
                    .try_into()
                    .map_err(|_| solana_program::program_error::ProgramError::InvalidAccountData)?,
            ),
        })
    }
}

pub fn find_rent_pda() -> (solana_program::pubkey::Pubkey, u8) {
    solana_program::pubkey::Pubkey::find_program_address(&[b"rent"], &ESPL_TOKEN_PROGRAM_ID)
}

pub fn find_lamports_pda(
    payer: &solana_program::pubkey::Pubkey,
    destination: &solana_program::pubkey::Pubkey,
    salt: &[u8; 32],
) -> (solana_program::pubkey::Pubkey, u8) {
    solana_program::pubkey::Pubkey::find_program_address(
        &[b"lamports", payer.as_ref(), destination.as_ref(), salt],
        &ESPL_TOKEN_PROGRAM_ID,
    )
}

pub fn find_vault_ata(
    mint: &solana_program::pubkey::Pubkey,
    vault: &solana_program::pubkey::Pubkey,
) -> solana_program::pubkey::Pubkey {
    find_associated_token_address_with_bump(vault, mint, &TOKEN_PROGRAM_ID).0
}

pub fn find_stash_pda(
    user: &solana_program::pubkey::Pubkey,
    mint: &solana_program::pubkey::Pubkey,
) -> (solana_program::pubkey::Pubkey, u8) {
    solana_program::pubkey::Pubkey::find_program_address(
        &[b"stash", user.as_ref(), mint.as_ref()],
        &ESPL_TOKEN_PROGRAM_ID,
    )
}

pub fn find_stash_ata(
    user: &solana_program::pubkey::Pubkey,
    mint: &solana_program::pubkey::Pubkey,
    token_program: &solana_program::pubkey::Pubkey,
) -> (solana_program::pubkey::Pubkey, u8) {
    let (stash_pda, _stash_bump) = find_stash_pda(user, mint);
    find_associated_token_address_with_bump(&stash_pda, mint, token_program)
}

pub fn find_shuttle_ephemeral_ata(
    owner: &solana_program::pubkey::Pubkey,
    mint: &solana_program::pubkey::Pubkey,
    shuttle_id: u32,
) -> (solana_program::pubkey::Pubkey, u8) {
    solana_program::pubkey::Pubkey::find_program_address(
        &[owner.as_ref(), mint.as_ref(), &shuttle_id.to_le_bytes()],
        &ESPL_TOKEN_PROGRAM_ID,
    )
}

pub fn find_shuttle_ata(
    shuttle_ephemeral_ata: &solana_program::pubkey::Pubkey,
    mint: &solana_program::pubkey::Pubkey,
) -> (solana_program::pubkey::Pubkey, u8) {
    solana_program::pubkey::Pubkey::find_program_address(
        &[shuttle_ephemeral_ata.as_ref(), mint.as_ref()],
        &ESPL_TOKEN_PROGRAM_ID,
    )
}

pub fn find_shuttle_wallet_ata(
    mint: &solana_program::pubkey::Pubkey,
    shuttle_ephemeral_ata: &solana_program::pubkey::Pubkey,
) -> solana_program::pubkey::Pubkey {
    find_associated_token_address_with_bump(shuttle_ephemeral_ata, mint, &TOKEN_PROGRAM_ID).0
}

pub fn find_transfer_queue(
    mint: &solana_program::pubkey::Pubkey,
    validator: &solana_program::pubkey::Pubkey,
) -> (solana_program::pubkey::Pubkey, u8) {
    solana_program::pubkey::Pubkey::find_program_address(
        &[b"queue", mint.as_ref(), validator.as_ref()],
        &ESPL_TOKEN_PROGRAM_ID,
    )
}

pub fn find_transfer_queue_ephemeral_ata(
    mint: &solana_program::pubkey::Pubkey,
    validator: &solana_program::pubkey::Pubkey,
) -> (solana_program::pubkey::Pubkey, u8) {
    let (queue, _queue_bump) = find_transfer_queue(mint, validator);
    solana_program::pubkey::Pubkey::find_program_address(
        &[queue.as_ref(), mint.as_ref()],
        &ESPL_TOKEN_PROGRAM_ID,
    )
}

pub fn find_transfer_queue_vault_ata(
    mint: &solana_program::pubkey::Pubkey,
    validator: &solana_program::pubkey::Pubkey,
    token_program: &solana_program::pubkey::Pubkey,
) -> solana_program::pubkey::Pubkey {
    let (queue, _queue_bump) = find_transfer_queue(mint, validator);
    find_associated_token_address_with_bump(&queue, mint, token_program).0
}

pub fn find_transfer_queue_refill_state(
    queue: &solana_program::pubkey::Pubkey,
) -> (solana_program::pubkey::Pubkey, u8) {
    solana_program::pubkey::Pubkey::find_program_address(
        &[b"queue-refill", queue.as_ref()],
        &ESPL_TOKEN_PROGRAM_ID,
    )
}

pub fn find_transfer_group_receipt(
    queue: &solana_program::pubkey::Pubkey,
    source: &solana_program::pubkey::Pubkey,
    group_id: u32,
) -> (solana_program::pubkey::Pubkey, u8) {
    assert!(group_id <= 0x00FF_FFFF, "group_id must fit in 24 bits");
    let group_id_bytes = group_id.to_le_bytes();
    solana_program::pubkey::Pubkey::find_program_address(
        &[
            b"group-receipt",
            queue.as_ref(),
            source.as_ref(),
            &group_id_bytes[..3],
        ],
        &ESPL_TOKEN_PROGRAM_ID,
    )
}

pub fn find_hydra_crank_pda(
    stash_pda: &solana_program::pubkey::Pubkey,
    shuttle_id: u32,
) -> (solana_program::pubkey::Pubkey, u8) {
    solana_program::pubkey::Pubkey::find_program_address(
        &[b"crank", &hydra_seed(stash_pda, shuttle_id)],
        &HYDRA_PROGRAM_ID,
    )
}

fn hydra_seed(stash_pda: &solana_program::pubkey::Pubkey, shuttle_id: u32) -> [u8; 32] {
    let mut seed = stash_pda.to_bytes();
    seed[..4].copy_from_slice(&shuttle_id.to_le_bytes());
    seed
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_find_transfer_group_receipt_uses_24_bit_group_id_seed() {
        let queue = solana_program::pubkey::Pubkey::new_from_array([1; 32]);
        let source = solana_program::pubkey::Pubkey::new_from_array([2; 32]);
        let group_id: u32 = 0x00FF_FFFF;
        let group_id_bytes = group_id.to_le_bytes();

        let receipt = find_transfer_group_receipt(&queue, &source, group_id);
        let expected = solana_program::pubkey::Pubkey::find_program_address(
            &[
                b"group-receipt",
                queue.as_ref(),
                source.as_ref(),
                &group_id_bytes[..3],
            ],
            &ESPL_TOKEN_PROGRAM_ID,
        );

        assert_eq!(receipt, expected);
    }

    #[test]
    #[should_panic(expected = "group_id must fit in 24 bits")]
    fn test_find_transfer_group_receipt_rejects_non_24_bit_group_id() {
        let queue = solana_program::pubkey::Pubkey::new_from_array([1; 32]);
        let source = solana_program::pubkey::Pubkey::new_from_array([2; 32]);

        find_transfer_group_receipt(&queue, &source, 0x0100_0000);
    }
}

pub(crate) fn find_associated_token_address_with_bump(
    wallet: &solana_program::pubkey::Pubkey,
    mint: &solana_program::pubkey::Pubkey,
    token_program: &solana_program::pubkey::Pubkey,
) -> (solana_program::pubkey::Pubkey, u8) {
    solana_program::pubkey::Pubkey::find_program_address(
        &[wallet.as_ref(), token_program.as_ref(), mint.as_ref()],
        &ASSOCIATED_TOKEN_PROGRAM_ID,
    )
}
