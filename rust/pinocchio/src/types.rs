use core::mem::size_of;
use pinocchio::{
    program_error::ProgramError,
    pubkey::Pubkey,
    pubkey::{MAX_SEEDS, MAX_SEED_LEN},
};

pub const MAX_DELEGATE_ACCOUNT_ARGS_SIZE: usize = size_of::<u32>() // commit_frequency_ms
    + size_of::<u32>() // seeds length
    + MAX_SEEDS * (size_of::<u32>() + MAX_SEED_LEN) // seeds
    + 1 + size_of::<Pubkey>(); // validator

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

impl DelegateAccountArgs<'_> {
    pub fn try_to_slice<'b>(&self, data: &'b mut [u8]) -> Result<&'b [u8], ProgramError> {
        if self.seeds.len() >= MAX_SEEDS {
            return Err(ProgramError::InvalidArgument);
        }

        for seed in self.seeds {
            if seed.len() > MAX_SEED_LEN {
                return Err(ProgramError::InvalidArgument);
            }
        }

        if data.len() != MAX_DELEGATE_ACCOUNT_ARGS_SIZE {
            return Err(ProgramError::InvalidArgument);
        }

        let mut offset = 0;

        // Serialize commit_frequency_ms (4 bytes)
        data[offset..offset + 4].copy_from_slice(&self.commit_frequency_ms.to_le_bytes());
        offset += 4;

        // Serialize seeds length (4 bytes)
        data[offset..offset + 4].copy_from_slice(&(self.seeds.len() as u32).to_le_bytes());
        offset += 4;

        // Serialize each seed
        for seed in self.seeds {
            data[offset..offset + 4].copy_from_slice(&(seed.len() as u32).to_le_bytes());
            offset += 4;
            data[offset..offset + seed.len()].copy_from_slice(seed);
            offset += seed.len();
        }

        match &self.validator {
            Some(pubkey) => {
                data[offset] = 1;
                offset += 1;
                data[offset..offset + 32].copy_from_slice(pubkey.as_ref());
                offset += 32;
            }
            None => {
                data[offset] = 0;
                offset += 1;
            }
        }

        Ok(&data[..offset])
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
