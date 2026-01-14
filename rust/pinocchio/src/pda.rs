use crate::seeds::Seed;
use pinocchio::Address;

#[cfg(not(any(
    target_os = "solana",
    target_arch = "bpf",
    feature = "address-find-program-address"
)))]
use solana_pubkey::Pubkey;

#[cfg(any(
    target_os = "solana",
    target_arch = "bpf",
    feature = "address-find-program-address"
))]
pub(crate) fn find_program_address(seeds: &[&[u8]], program_id: &Address) -> (Address, u8) {
    Address::find_program_address(seeds, program_id)
}

#[cfg(not(any(
    target_os = "solana",
    target_arch = "bpf",
    feature = "address-find-program-address"
)))]
pub(crate) fn find_program_address(seeds: &[&[u8]], program_id: &Address) -> (Address, u8) {
    let program_pubkey = Pubkey::from(*program_id.as_array());
    let (pda, bump) = Pubkey::find_program_address(seeds, &program_pubkey);
    (Address::new_from_array(pda.to_bytes()), bump)
}

/// Find a PDA from a typed `Seed`
fn find_seed_pda(seed: &Seed, program_id: &Address) -> Address {
    let mut buf: [&[u8]; 3] = [&[]; 3];
    let mut index_buf = [0u8; 1];
    let seeds = seed.fill_seed_slice(&mut buf, &mut index_buf);
    let (pda, _bump) = find_program_address(seeds, program_id);
    pda
}

// Specialized functions calling the generic one
pub fn delegation_record_pda_from_delegated_account(delegated: &Address) -> Address {
    find_seed_pda(&Seed::Delegation(delegated), crate::id())
}

pub fn delegation_metadata_pda_from_delegated_account(delegated: &Address) -> Address {
    find_seed_pda(&Seed::DelegationMetadata(delegated), crate::id())
}

pub fn commit_state_pda_from_delegated_account(delegated: &Address) -> Address {
    find_seed_pda(&Seed::CommitState(delegated), crate::id())
}

pub fn commit_record_pda_from_delegated_account(delegated: &Address) -> Address {
    find_seed_pda(&Seed::CommitRecord(delegated), crate::id())
}

pub fn delegate_buffer_pda_from_delegated_account_and_owner_program(
    delegated: &Address,
    owner_program: &Address,
) -> Address {
    find_seed_pda(&Seed::Buffer(delegated), owner_program)
}

pub fn undelegate_buffer_pda_from_delegated_account(delegated: &Address) -> Address {
    find_seed_pda(&Seed::UndelegateBuffer(delegated), crate::id())
}

pub fn fees_vault_pda() -> Address {
    find_seed_pda(&Seed::FeesVault, crate::id())
}

pub fn validator_fees_vault_pda_from_validator(validator: &Address) -> Address {
    find_seed_pda(&Seed::ValidatorFeesVault(validator), crate::id())
}

pub fn program_config_from_program_id(program_id: &Address) -> Address {
    find_seed_pda(&Seed::ProgramConfig(program_id), crate::id())
}

pub fn ephemeral_balance_pda_from_payer(payer: &Address, index: u8) -> Address {
    find_seed_pda(&Seed::EphemeralBalance { payer, index }, crate::id())
}
