use core::mem::size_of;
use pinocchio::{address::MAX_SEEDS, address::MAX_SEED_LEN, error::ProgramError, Address};

pub const MAX_DELEGATE_ACCOUNT_ARGS_SIZE: usize = size_of::<u32>() // commit_frequency_ms
    + size_of::<u32>() // seeds length
    + MAX_SEEDS * (size_of::<u32>() + MAX_SEED_LEN) // seeds
    + 1 + size_of::<Address>(); // validator

/// Arbitrary limit on the size of the post-delegation actions.
/// Can't be accurate because instructions sizes vary and some data is encrypted.
pub const MAX_POST_DELEGATION_ACTIONS_SIZE: usize = 2000;

const ACCOUNT_INDEX_MASK: u8 = 0b0011_1111;
const SIGNER_MASK: u8 = 0b0100_0000;
const WRITABLE_MASK: u8 = 0b1000_0000;

///
/// MAX_PUBKEYS = 64
///
pub const MAX_PUBKEYS: u8 = ACCOUNT_INDEX_MASK + 1;

/// Compact account meta packed into one byte.
/// Bits `0..=5` encode the pubkey-table index (`0..MAX_PUBKEYS-1`).
/// Bit `6` is `is_signer`, and bit `7` is `is_writable`.
#[derive(Clone, Copy, Debug)]
pub struct CompactAccountMeta(u8);

impl CompactAccountMeta {
    pub fn new(index: u8, is_signer: bool) -> Self {
        Self::try_new(index, is_signer, true).expect("index is out of range")
    }
    pub fn new_readonly(index: u8, is_signer: bool) -> Self {
        Self::try_new(index, is_signer, false).expect("index is out of range")
    }

    pub fn try_new(index: u8, is_signer: bool, is_writable: bool) -> Option<Self> {
        if index >= MAX_PUBKEYS {
            return None;
        }
        let mut packed = index;
        if is_signer {
            packed |= SIGNER_MASK;
        }
        if is_writable {
            packed |= WRITABLE_MASK;
        }
        Some(Self(packed))
    }

    pub fn key(self) -> u8 {
        self.0 & ACCOUNT_INDEX_MASK
    }

    pub fn is_signer(self) -> bool {
        (self.0 & SIGNER_MASK) != 0
    }

    pub fn is_writable(self) -> bool {
        (self.0 & WRITABLE_MASK) != 0
    }

    pub fn set_index(&mut self, new_index: u8) {
        *self = Self::try_new(new_index, self.is_signer(), self.is_writable())
            .expect("index is out of range");
    }

    pub fn to_byte(self) -> u8 {
        self.0
    }

    pub fn from_byte(value: u8) -> Option<Self> {
        Self::try_new(
            value & ACCOUNT_INDEX_MASK,
            (value & SIGNER_MASK) != 0,
            (value & WRITABLE_MASK) != 0,
        )
    }
}

#[derive(Clone, Copy, Debug)]
pub struct EncryptableAccountMeta {
    pub account_meta: CompactAccountMeta,
    pub is_encryptable: bool,
}

#[derive(Clone, Debug)]
pub struct MaybeEncryptedInstruction<'a> {
    pub program_id: u8,

    pub accounts: &'a [MaybeEncryptedAccountMeta<'a>],

    pub data: MaybeEncryptedIxData<'a>,
}

#[derive(Clone, Debug)]
pub enum MaybeEncryptedPubkey<'a> {
    ClearText(Address),
    Encrypted(EncryptedBuffer<'a>),
}

#[derive(Clone, Debug)]
pub enum MaybeEncryptedAccountMeta<'a> {
    ClearText(CompactAccountMeta),
    Encrypted(EncryptedBuffer<'a>),
}

#[derive(Clone, Debug)]
pub struct MaybeEncryptedIxData<'a> {
    pub prefix: &'a [u8],
    pub suffix: EncryptedBuffer<'a>,
}

#[derive(Clone, Debug, Default)]
pub struct EncryptedBuffer<'a>(&'a [u8]);

#[derive(Debug)]
pub struct PostDelegationActions<'a> {
    pub signers: &'a [Address],

    pub non_signers: &'a [MaybeEncryptedPubkey<'a>],

    pub instructions: &'a [MaybeEncryptedInstruction<'a>],
}

impl PostDelegationActions<'_> {
    pub fn try_to_slice<'b>(&self, data: &'b mut [u8]) -> Result<&'b [u8], ProgramError> {
        if data.len() > MAX_POST_DELEGATION_ACTIONS_SIZE {
            return Err(ProgramError::InvalidArgument);
        }

        let mut offset = 0;

        // Serialize signers length (4 bytes)
        data[offset..offset + 4].copy_from_slice(&(self.signers.len() as u32).to_le_bytes());
        offset += 4;

        // Serialize signers
        for signer in self.signers {
            data[offset..offset + 32].copy_from_slice(signer.as_ref());
            offset += 32;
        }

        // Serialize non_signers length (4 bytes)
        data[offset..offset + 4].copy_from_slice(&(self.non_signers.len() as u32).to_le_bytes());
        offset += 4;

        // Serialize non_signers
        for non_signer in self.non_signers {
            match non_signer {
                MaybeEncryptedPubkey::ClearText(pubkey) => {
                    data[offset..offset + 1].copy_from_slice(&[0]);
                    offset += 1;
                    data[offset..offset + 32].copy_from_slice(pubkey.as_ref());
                    offset += 32;
                }
                MaybeEncryptedPubkey::Encrypted(encrypted_buffer) => {
                    data[offset..offset + 1].copy_from_slice(&[1]);
                    offset += 1;
                    data[offset..offset + 4]
                        .copy_from_slice(&(encrypted_buffer.0.len() as u32).to_le_bytes());
                    offset += 4;
                    data[offset..offset + encrypted_buffer.0.len()]
                        .copy_from_slice(encrypted_buffer.0);
                    offset += encrypted_buffer.0.len();
                }
            }
        }

        // Serialize instructions length (4 bytes)
        data[offset..offset + 4].copy_from_slice(&(self.instructions.len() as u32).to_le_bytes());
        offset += 4;

        // Serialize instructions
        for instruction in self.instructions {
            data[offset..offset + 1].copy_from_slice(&[instruction.program_id]);
            offset += 1;

            // Serialize accounts length (4 bytes)
            data[offset..offset + 4]
                .copy_from_slice(&(instruction.accounts.len() as u32).to_le_bytes());
            offset += 4;

            // Serialize accounts
            for account in instruction.accounts {
                match account {
                    MaybeEncryptedAccountMeta::ClearText(account_meta) => {
                        data[offset..offset + 1].copy_from_slice(&[0]);
                        offset += 1;
                        data[offset..offset + 1].copy_from_slice(&[account_meta.to_byte()]);
                        offset += 1;
                    }
                    MaybeEncryptedAccountMeta::Encrypted(encrypted_buffer) => {
                        data[offset..offset + 1].copy_from_slice(&[1]);
                        offset += 1;
                        data[offset..offset + 4]
                            .copy_from_slice(&(encrypted_buffer.0.len() as u32).to_le_bytes());
                        offset += 4;
                        data[offset..offset + encrypted_buffer.0.len()]
                            .copy_from_slice(encrypted_buffer.0);
                        offset += encrypted_buffer.0.len();
                    }
                }
            }

            // Serialize data length (4 bytes)
            data[offset..offset + 4]
                .copy_from_slice(&(instruction.data.prefix.len() as u32).to_le_bytes());
            offset += 4;

            // Serialize data prefix
            data[offset..offset + instruction.data.prefix.len()]
                .copy_from_slice(instruction.data.prefix);
            offset += instruction.data.prefix.len();

            // Serialize data suffix length (4 bytes)
            data[offset..offset + 4]
                .copy_from_slice(&(instruction.data.suffix.0.len() as u32).to_le_bytes());
            offset += 4;

            // Serialize data suffix
            data[offset..offset + instruction.data.suffix.0.len()]
                .copy_from_slice(instruction.data.suffix.0);
            offset += instruction.data.suffix.0.len();
        }

        Ok(&data[..offset])
    }
}

#[derive(Debug)]
pub struct DelegateAccountArgs<'a> {
    pub commit_frequency_ms: u32,
    pub seeds: &'a [&'a [u8]],
    pub validator: Option<Address>,
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
    pub validator: Option<Address>,
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
    use alloc::vec;
    use alloc::vec::Vec;
    use dlp_api::dlp::{
        args::{
            EncryptedBuffer as DlpEncryptedBuffer,
            MaybeEncryptedAccountMeta as DlpMaybeEncryptedAccountMeta,
            MaybeEncryptedInstruction as DlpMaybeEncryptedInstruction,
            MaybeEncryptedIxData as DlpMaybeEncryptedIxData,
            MaybeEncryptedPubkey as DlpMaybeEncryptedPubkey,
            PostDelegationActions as DlpPostDelegationActions,
        },
        compact::AccountMeta as DlpAccountMeta,
    };
    use magicblock_magic_program_api::Pubkey;

    use super::*;

    pub fn build_actions_from_dlp_actions<'a>(
        dlp_actions: &'a DlpPostDelegationActions,
        mut data: &'a mut [u8],
    ) -> usize {
        let signers_vec = dlp_actions
            .signers
            .iter()
            .map(|pubkey| Address::new_from_array(pubkey.to_bytes()))
            .collect::<Vec<_>>();
        let non_signers_vec = dlp_actions
            .non_signers
            .iter()
            .map(|acc| match acc {
                DlpMaybeEncryptedPubkey::ClearText(pubkey) => {
                    MaybeEncryptedPubkey::ClearText(Address::new_from_array(pubkey.to_bytes()))
                }
                DlpMaybeEncryptedPubkey::Encrypted(encrypted_buffer) => {
                    MaybeEncryptedPubkey::Encrypted(EncryptedBuffer(encrypted_buffer.as_bytes()))
                }
            })
            .collect::<Vec<_>>();
        let instruction_accounts_vec = dlp_actions
            .instructions
            .iter()
            .map(|instruction| {
                instruction
                    .accounts
                    .iter()
                    .map(|account| match account {
                        DlpMaybeEncryptedAccountMeta::ClearText(account) => {
                            MaybeEncryptedAccountMeta::ClearText(CompactAccountMeta::new(
                                account.key(),
                                account.is_signer(),
                            ))
                        }
                        DlpMaybeEncryptedAccountMeta::Encrypted(encrypted_buffer) => {
                            MaybeEncryptedAccountMeta::Encrypted(EncryptedBuffer(
                                encrypted_buffer.as_bytes(),
                            ))
                        }
                    })
                    .collect::<Vec<_>>()
            })
            .collect::<Vec<_>>();
        let instructions_vec = dlp_actions
            .instructions
            .iter()
            .enumerate()
            .map(|(i, instruction)| MaybeEncryptedInstruction {
                program_id: instruction.program_id,
                accounts: instruction_accounts_vec[i].as_slice(),
                data: MaybeEncryptedIxData {
                    prefix: &instruction.data.prefix,
                    suffix: EncryptedBuffer(instruction.data.suffix.as_bytes()),
                },
            })
            .collect::<Vec<_>>();

        let actions = PostDelegationActions {
            signers: &signers_vec.as_slice(),
            non_signers: &non_signers_vec.as_slice(),
            instructions: &instructions_vec.as_slice(),
        };

        let slice = actions.try_to_slice(&mut data).unwrap();
        slice.len()
    }

    #[test]
    fn test_serialize_empty_post_delegation_actions() {
        let signers = vec![];
        let non_signers = vec![];
        let instructions = vec![];

        let dlp_actions = DlpPostDelegationActions {
            signers,
            non_signers,
            instructions,
        };

        let mut data = [0u8; MAX_POST_DELEGATION_ACTIONS_SIZE];
        let slice_len = build_actions_from_dlp_actions(&dlp_actions, &mut data);

        let dlp_actions_vec = borsh::to_vec(&dlp_actions).unwrap();
        assert_eq!(&data[..slice_len], dlp_actions_vec.as_slice());
    }

    #[test]
    fn test_serialize_cleartext_post_delegation_actions() {
        let signers = vec![
            Pubkey::new_from_array([1; 32]),
            Pubkey::new_from_array([2; 32]),
        ];
        let non_signers = vec![
            DlpMaybeEncryptedPubkey::ClearText(Pubkey::new_from_array([3; 32])),
            DlpMaybeEncryptedPubkey::ClearText(Pubkey::new_from_array([4; 32])),
        ];
        let dlp_instructions = vec![DlpMaybeEncryptedInstruction {
            program_id: 3,
            accounts: vec![
                DlpMaybeEncryptedAccountMeta::ClearText(DlpAccountMeta::new(0, true)),
                DlpMaybeEncryptedAccountMeta::ClearText(DlpAccountMeta::new(1, false)),
            ],
            data: DlpMaybeEncryptedIxData {
                prefix: vec![0; 32],
                suffix: DlpEncryptedBuffer::new(vec![]),
            },
        }];

        let dlp_actions = DlpPostDelegationActions {
            signers,
            non_signers,
            instructions: dlp_instructions.clone(),
        };

        let mut data = [0u8; MAX_POST_DELEGATION_ACTIONS_SIZE];
        let slice_len = build_actions_from_dlp_actions(&dlp_actions, &mut data);

        let dlp_actions_vec = borsh::to_vec(&dlp_actions).unwrap();
        assert_eq!(&data[..slice_len], dlp_actions_vec.as_slice());
    }

    #[test]
    fn test_serialize_encrypted_post_delegation_actions() {
        let signers = vec![
            Pubkey::new_from_array([1; 32]),
            Pubkey::new_from_array([2; 32]),
        ];
        let non_signers = vec![
            DlpMaybeEncryptedPubkey::Encrypted(DlpEncryptedBuffer::new(vec![3; 32])),
            DlpMaybeEncryptedPubkey::Encrypted(DlpEncryptedBuffer::new(vec![5; 5])),
        ];
        let dlp_instructions = vec![DlpMaybeEncryptedInstruction {
            program_id: 3,
            accounts: vec![
                DlpMaybeEncryptedAccountMeta::Encrypted(DlpEncryptedBuffer::new(vec![4; 5])),
                DlpMaybeEncryptedAccountMeta::Encrypted(DlpEncryptedBuffer::new(vec![5; 5])),
            ],
            data: DlpMaybeEncryptedIxData {
                prefix: vec![],
                suffix: DlpEncryptedBuffer::new(vec![8; 5]),
            },
        }];

        let dlp_actions = DlpPostDelegationActions {
            signers,
            non_signers,
            instructions: dlp_instructions.clone(),
        };

        let mut data = [0u8; MAX_POST_DELEGATION_ACTIONS_SIZE];
        let slice_len = build_actions_from_dlp_actions(&dlp_actions, &mut data);

        let dlp_actions_vec = borsh::to_vec(&dlp_actions).unwrap();
        assert_eq!(&data[..slice_len], dlp_actions_vec.as_slice());
    }

    #[test]
    fn test_serialize_partially_encrypted_post_delegation_actions() {
        let signers = vec![
            Pubkey::new_from_array([1; 32]),
            Pubkey::new_from_array([2; 32]),
        ];
        let non_signers = vec![
            DlpMaybeEncryptedPubkey::ClearText(Pubkey::new_from_array([3; 32])),
            DlpMaybeEncryptedPubkey::Encrypted(DlpEncryptedBuffer::new(vec![5; 5])),
        ];
        let dlp_instructions = vec![DlpMaybeEncryptedInstruction {
            program_id: 3,
            accounts: vec![
                DlpMaybeEncryptedAccountMeta::ClearText(DlpAccountMeta::new(0, true)),
                DlpMaybeEncryptedAccountMeta::Encrypted(DlpEncryptedBuffer::new(vec![5; 5])),
            ],
            data: DlpMaybeEncryptedIxData {
                prefix: vec![1, 2, 3, 4, 5],
                suffix: DlpEncryptedBuffer::new(vec![8; 5]),
            },
        }];

        let dlp_actions = DlpPostDelegationActions {
            signers: signers,
            non_signers: non_signers,
            instructions: dlp_instructions.clone(),
        };

        let mut data = [0u8; MAX_POST_DELEGATION_ACTIONS_SIZE];
        let slice_len = build_actions_from_dlp_actions(&dlp_actions, &mut data);

        let dlp_actions_vec = borsh::to_vec(&dlp_actions).unwrap();
        assert_eq!(&data[..slice_len], dlp_actions_vec.as_slice());
    }
}
