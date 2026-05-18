use crate::{
    compat,
    consts::{
        ASSOCIATED_TOKEN_PROGRAM_ID, ESPL_TOKEN_PROGRAM_ID, HYDRA_PROGRAM_ID, TOKEN_PROGRAM_ID,
    },
};

/// Internal representation of a token account data.
#[repr(C)]
pub struct EphemeralAta {
    /// The owner of the eata
    pub owner: compat::Pubkey,
    /// The mint associated with this account
    pub mint: compat::Pubkey,
    /// The amount of tokens this account holds.
    pub amount: u64,
}

impl EphemeralAta {
    pub const LEN: usize = 32 + 32 + 8;

    pub fn find_pda(user: &compat::Pubkey, mint: &compat::Pubkey) -> (compat::Pubkey, u8) {
        compat::Pubkey::find_program_address(
            &[user.as_ref(), mint.as_ref()],
            &ESPL_TOKEN_PROGRAM_ID,
        )
    }

    pub fn try_from_bytes(bytes: &[u8]) -> Result<Self, compat::ProgramError> {
        if bytes.len() != Self::LEN {
            return Err(compat::ProgramError::InvalidAccountData);
        }
        Ok(Self {
            owner: compat::Pubkey::new_from_array(
                bytes[0..32]
                    .try_into()
                    .map_err(|_| compat::ProgramError::InvalidAccountData)?,
            ),
            mint: compat::Pubkey::new_from_array(
                bytes[32..64]
                    .try_into()
                    .map_err(|_| compat::ProgramError::InvalidAccountData)?,
            ),
            amount: u64::from_le_bytes(
                bytes[64..72]
                    .try_into()
                    .map_err(|_| compat::ProgramError::InvalidAccountData)?,
            ),
        })
    }
}

/// Internal representation of a global vault for a specific mint.
#[repr(C)]
pub struct GlobalVault {
    /// The mint associated with this vault
    pub mint: compat::Pubkey,
}

impl GlobalVault {
    pub const LEN: usize = 32;

    pub fn find_pda(mint: &compat::Pubkey) -> (compat::Pubkey, u8) {
        compat::Pubkey::find_program_address(&[mint.as_ref()], &ESPL_TOKEN_PROGRAM_ID)
    }

    pub fn try_from_bytes(bytes: &[u8]) -> Result<Self, compat::ProgramError> {
        if bytes.len() != Self::LEN {
            return Err(compat::ProgramError::InvalidAccountData);
        }
        Ok(Self {
            mint: compat::Pubkey::new_from_array(
                bytes[0..32]
                    .try_into()
                    .map_err(|_| compat::ProgramError::InvalidAccountData)?,
            ),
        })
    }
}

pub fn find_rent_pda() -> (compat::Pubkey, u8) {
    compat::Pubkey::find_program_address(&[b"rent"], &ESPL_TOKEN_PROGRAM_ID)
}

pub fn find_lamports_pda(
    payer: &compat::Pubkey,
    destination: &compat::Pubkey,
    salt: &[u8; 32],
) -> (compat::Pubkey, u8) {
    compat::Pubkey::find_program_address(
        &[b"lamports", payer.as_ref(), destination.as_ref(), salt],
        &ESPL_TOKEN_PROGRAM_ID,
    )
}

pub fn find_vault_ata(mint: &compat::Pubkey, vault: &compat::Pubkey) -> compat::Pubkey {
    find_associated_token_address_with_bump(vault, mint, &TOKEN_PROGRAM_ID).0
}

pub fn find_stash_pda(user: &compat::Pubkey, mint: &compat::Pubkey) -> (compat::Pubkey, u8) {
    compat::Pubkey::find_program_address(
        &[b"stash", user.as_ref(), mint.as_ref()],
        &ESPL_TOKEN_PROGRAM_ID,
    )
}

pub fn find_stash_ata(
    user: &compat::Pubkey,
    mint: &compat::Pubkey,
    token_program: &compat::Pubkey,
) -> (compat::Pubkey, u8) {
    let (stash_pda, _stash_bump) = find_stash_pda(user, mint);
    find_associated_token_address_with_bump(&stash_pda, mint, token_program)
}

pub fn find_shuttle_ephemeral_ata(
    owner: &compat::Pubkey,
    mint: &compat::Pubkey,
    shuttle_id: u32,
) -> (compat::Pubkey, u8) {
    compat::Pubkey::find_program_address(
        &[owner.as_ref(), mint.as_ref(), &shuttle_id.to_le_bytes()],
        &ESPL_TOKEN_PROGRAM_ID,
    )
}

pub fn find_shuttle_ata(
    shuttle_ephemeral_ata: &compat::Pubkey,
    mint: &compat::Pubkey,
) -> (compat::Pubkey, u8) {
    compat::Pubkey::find_program_address(
        &[shuttle_ephemeral_ata.as_ref(), mint.as_ref()],
        &ESPL_TOKEN_PROGRAM_ID,
    )
}

pub fn find_shuttle_wallet_ata(
    mint: &compat::Pubkey,
    shuttle_ephemeral_ata: &compat::Pubkey,
) -> compat::Pubkey {
    find_associated_token_address_with_bump(shuttle_ephemeral_ata, mint, &TOKEN_PROGRAM_ID).0
}

pub fn find_transfer_queue(
    mint: &compat::Pubkey,
    validator: &compat::Pubkey,
) -> (compat::Pubkey, u8) {
    compat::Pubkey::find_program_address(
        &[b"queue", mint.as_ref(), validator.as_ref()],
        &ESPL_TOKEN_PROGRAM_ID,
    )
}

pub fn find_transfer_queue_refill_state(queue: &compat::Pubkey) -> (compat::Pubkey, u8) {
    compat::Pubkey::find_program_address(&[b"queue-refill", queue.as_ref()], &ESPL_TOKEN_PROGRAM_ID)
}

pub fn find_hydra_crank_pda(stash_pda: &compat::Pubkey, shuttle_id: u32) -> (compat::Pubkey, u8) {
    compat::Pubkey::find_program_address(
        &[b"crank", &hydra_seed(stash_pda, shuttle_id)],
        &HYDRA_PROGRAM_ID,
    )
}

fn hydra_seed(stash_pda: &compat::Pubkey, shuttle_id: u32) -> [u8; 32] {
    let mut seed = stash_pda.to_bytes();
    seed[..4].copy_from_slice(&shuttle_id.to_le_bytes());
    seed
}

pub(crate) fn find_associated_token_address_with_bump(
    wallet: &compat::Pubkey,
    mint: &compat::Pubkey,
    token_program: &compat::Pubkey,
) -> (compat::Pubkey, u8) {
    compat::Pubkey::find_program_address(
        &[wallet.as_ref(), token_program.as_ref(), mint.as_ref()],
        &ASSOCIATED_TOKEN_PROGRAM_ID,
    )
}
