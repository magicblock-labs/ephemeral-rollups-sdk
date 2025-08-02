
use pinocchio::pubkey::Pubkey;

#[derive(Debug)]
pub struct DelegateAccountArgs<'a> {
    pub commit_frequency_ms: u32,
    pub seeds: &'a [&'a [u8]],
    pub validator: Option<Pubkey>,
}

impl<'a> Default for DelegateAccountArgs<'a> {
    fn default() -> Self {
        DelegateAccountArgs {
            commit_frequency_ms: u32::MAX,
            seeds: &[],
            validator: None,
        }
    }
}

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
