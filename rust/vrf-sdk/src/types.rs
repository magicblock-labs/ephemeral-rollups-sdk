use borsh::{BorshDeserialize, BorshSerialize};

#[derive(BorshSerialize, BorshDeserialize, Debug, PartialEq, Default)]
#[borsh(crate = "borsh")]
pub struct RequestRandomness {
    pub caller_seed: [u8; 32],
    pub callback_program_id: solana_program::pubkey::Pubkey,
    pub callback_discriminator: Vec<u8>,
    pub callback_accounts_metas: Vec<SerializableAccountMeta>,
    pub callback_args: Vec<u8>,
}

impl RequestRandomness {
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = vec![3, 0, 0, 0, 0, 0, 0, 0];
        self.serialize(&mut bytes).unwrap();
        bytes
    }
}

#[derive(BorshSerialize, BorshDeserialize, Debug, PartialEq, Default, Clone)]
#[borsh(crate = "borsh")]
pub struct SerializableAccountMeta {
    pub pubkey: solana_program::pubkey::Pubkey,
    pub is_signer: bool,
    pub is_writable: bool,
}
