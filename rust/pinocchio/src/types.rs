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
    pub fn try_to_serialize(&self) -> Result<Vec<u8>, ProgramError> {
        let mut data_vec = Vec::new();

        // Serialize commit_frequency_ms
        data_vec.extend_from_slice(&self.commit_frequency_ms.to_le_bytes());

        // Serialize seeds count
        data_vec.extend_from_slice(&(self.seeds.len() as u32).to_le_bytes());

        // Serialize each seed
        for seed in self.seeds {
            data_vec.extend_from_slice(&(seed.len() as u32).to_le_bytes());
            data_vec.extend_from_slice(seed);
        }

        // Serialize validator
        match &self.validator {
            Some(pubkey) => {
                data_vec.push(1);
                data_vec.extend_from_slice(pubkey.as_ref());
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
