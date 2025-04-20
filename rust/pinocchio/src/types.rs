use borsh::{BorshDeserialize, BorshSerialize};
use pinocchio::pubkey::Pubkey;

#[derive(Debug, BorshSerialize, BorshDeserialize)]
pub struct DelegateAccountArgs {
    pub commit_frequency_ms: u32,
    pub seeds: Vec<Vec<u8>>,
    pub validator: Option<Pubkey>,
}

impl Default for DelegateAccountArgs {
    fn default() -> Self {
        DelegateAccountArgs {
            commit_frequency_ms: u32::MAX,
            seeds: vec![],
            validator: None,
        }
    }
}

#[derive(BorshSerialize, BorshDeserialize)]
pub struct DelegateConfig {
    pub commit_frequency_ms: u32,
    pub validator: Option<Pubkey>,
}

impl Default for DelegateConfig {
    fn default() -> Self {
        DelegateConfig {
            commit_frequency_ms: DelegateAccountArgs::default().commit_frequency_ms,
            validator: DelegateAccountArgs::default().validator,
        }
    }
}
