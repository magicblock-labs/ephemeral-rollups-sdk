use crate::compat;
use crate::compat::borsh;
use borsh::{BorshDeserialize, BorshSerialize};

#[derive(BorshSerialize, BorshDeserialize, Debug, PartialEq, Default)]
#[cfg_attr(
    not(feature = "backward-compat"),
    borsh(crate = "crate::compat::borsh")
)]
pub struct RequestRandomness {
    pub caller_seed: [u8; 32],
    pub callback_program_id: compat::Pubkey,
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
#[cfg_attr(
    not(feature = "backward-compat"),
    borsh(crate = "crate::compat::borsh")
)]
pub struct SerializableAccountMeta {
    pub pubkey: compat::Pubkey,
    pub is_signer: bool,
    pub is_writable: bool,
}
