use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::pubkey::Pubkey;

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

#[cfg(feature = "light")]
#[derive(Default, Debug, BorshSerialize, BorshDeserialize)]
pub struct DelegateCompressedArgs {
    /// The frequency at which the validator should commit the account data
    /// if no commit is triggered by the owning program
    pub commit_frequency_ms: u32,
    /// The seeds used to derive the PDA of the delegated account
    pub seeds: Vec<Vec<u8>>,
    /// The validator authority that is added to the delegation record
    pub validator: Option<Pubkey>,
    /// The proof of the account data
    pub proof: light_sdk::instruction::ValidityProof,
    /// The address tree info
    pub address_tree_info: light_sdk::instruction::PackedAddressTreeInfo,
    /// The output state tree index
    pub output_state_tree_index: u8,
    /// The account meta
    pub account_meta: light_sdk::instruction::account_meta::CompressedAccountMeta,
    /// The account data
    pub account_data: Vec<u8>,
}

#[cfg(feature = "idl-build")]
impl anchor_lang::IdlBuild for DelegateCompressedArgs {}

#[cfg(feature = "idl-build")]
impl anchor_lang::Discriminator for DelegateCompressedArgs {
    const DISCRIMINATOR: &'static [u8] = &[];
}
