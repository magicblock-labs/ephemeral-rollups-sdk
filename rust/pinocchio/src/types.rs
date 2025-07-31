use pinocchio::program_error::ProgramError;
use pinocchio::pubkey::Pubkey;

#[derive(Debug)]
pub struct DelegateAccountArgs<'a> {
    pub commit_frequency_ms: u32,
    pub seeds: &'a [&'a [u8]],
    pub validator: Option<Pubkey>,
}

impl Default for DelegateAccountArgs<'_> {
    fn default() -> Self {
        DelegateAccountArgs {
            commit_frequency_ms: u32::MAX,
            seeds: &[],
            validator: None,
        }
    }
}

impl<'a> DelegateAccountArgs<'a> {
    pub fn try_to_vec(&self) -> Result<Vec<u8>, ProgramError> {
        let mut data_vec = Vec::new();

        //Serialize commit_frequency_ms
        data_vec.extend(&self.commit_frequency_ms.to_le_bytes());

        //Serialize seeds
        data_vec.extend(&(self.seeds.len() as u32).to_le_bytes());
        for seed in &*self.seeds {
            data_vec.extend(&(seed.len() as u32).to_le_bytes());
            data_vec.extend(*seed);
        }
        // Serialize validator
        match &self.validator {
            Some(pubkey) => {
                data_vec.push(1);
                data_vec.extend(pubkey);
            }
            None => {
                data_vec.push(0);
            }
        }
        Ok(data_vec)
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
