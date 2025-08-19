use crate::seeds::Seed;
use pinocchio::pubkey;
use pinocchio::pubkey::Pubkey;

/// Generic DRY function to find a PDA from a typed `Seed`
fn find_seed_pda(seed: &Seed, program_id: &Pubkey) -> Pubkey {
    let mut buf: [&[u8]; 3] = [&[]; 3];
    let mut index_buf = [0u8; 1];
    let seeds = seed.fill_seed_slice(&mut buf, &mut index_buf);
    pubkey::find_program_address(seeds, program_id).0
}

// Specialized functions calling the generic one
pub fn delegation_record_pda_from_delegated_account(delegated: &Pubkey) -> Pubkey {
    find_seed_pda(&Seed::Delegation(delegated), &crate::id())
}

pub fn delegation_metadata_pda_from_delegated_account(delegated: &Pubkey) -> Pubkey {
    find_seed_pda(&Seed::DelegationMetadata(delegated), &crate::id())
}

pub fn commit_state_pda_from_delegated_account(delegated: &Pubkey) -> Pubkey {
    find_seed_pda(&Seed::CommitState(delegated), &crate::id())
}

pub fn commit_record_pda_from_delegated_account(delegated: &Pubkey) -> Pubkey {
    find_seed_pda(&Seed::CommitRecord(delegated), &crate::id())
}

pub fn delegate_buffer_pda_from_delegated_account_and_owner_program(
    delegated: &Pubkey,
    owner_program: &Pubkey,
) -> Pubkey {
    find_seed_pda(&Seed::Buffer(delegated), owner_program)
}

pub fn undelegate_buffer_pda_from_delegated_account(delegated: &Pubkey) -> Pubkey {
    find_seed_pda(&Seed::UndelegateBuffer(delegated), &crate::id())
}

pub fn fees_vault_pda() -> Pubkey {
    find_seed_pda(&Seed::FeesVault, &crate::id())
}

pub fn validator_fees_vault_pda_from_validator(validator: &Pubkey) -> Pubkey {
    find_seed_pda(&Seed::ValidatorFeesVault(validator), &crate::id())
}

pub fn program_config_from_program_id(program_id: &Pubkey) -> Pubkey {
    find_seed_pda(&Seed::ProgramConfig(program_id), &crate::id())
}

pub fn ephemeral_balance_pda_from_payer(payer: &Pubkey, index: u8) -> Pubkey {
    find_seed_pda(&Seed::EphemeralBalance { payer, index }, &crate::id())
}
