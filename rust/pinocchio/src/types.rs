use pinocchio::program_error::ProgramError;
use pinocchio::pubkey::{Pubkey, MAX_SEEDS, MAX_SEED_LEN};

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

impl<'a> DelegateAccountArgs<'a> {
    pub fn try_to_serialize(&self) -> Result<&[u8], ProgramError> {
        if self.seeds.len() >= MAX_SEEDS {
            return Err(ProgramError::InvalidArgument);
        }

        for seed in self.seeds {
            if seed.len() > MAX_SEED_LEN {
                return Err(ProgramError::InvalidArgument);
            }
        }

        let mut data = [0u8; MAX_DELEGATE_ACCOUNT_ARGS_SIZE];
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
        unsafe {
            // SAFETY: offset <= MAX_DELEGATE_ACCOUNT_ARGS_SIZE and we've written to data[..offset]
            Ok(core::slice::from_raw_parts(data.as_ptr(), offset))
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_delegate_account_args_serialization() {
        let args = DelegateAccountArgs {
            commit_frequency_ms: 1000,
            seeds: &["seed1".as_bytes(), "seed2".as_bytes()],
            validator: Some(Pubkey::from([1u8; 32])),
        };

        let serialized = args.try_to_serialize().unwrap();
        let expected_length = 4 // commit_frequency_ms(u32)
            + 4 // seeds count(u32)
            + 4 + 5 // seed1 length count(u32) + seed1 data length(u32)
            + 4 + 5 // seed2 length count(u32) + seed2 data length(u32)
            + 1 // validator presence flag (u8)
            + 32; // validator pubkey(Pubkey is 32 bytes)

        assert_eq!(serialized.len(), expected_length);

        assert_eq!(&serialized[0..4], &u32::to_le_bytes(1000));
        assert_eq!(&serialized[4..8], &u32::to_le_bytes(2));
        assert_eq!(&serialized[8..12], &u32::to_le_bytes(5));
        assert_eq!(&serialized[12..17], b"seed1");
        assert_eq!(&serialized[17..21], &u32::to_le_bytes(5));
        assert_eq!(&serialized[21..26], b"seed2");
        assert_eq!(serialized[26], 1); // validator presence flag
        assert_eq!(&serialized[27..59], &Pubkey::from([1u8; 32]));
    }
}
