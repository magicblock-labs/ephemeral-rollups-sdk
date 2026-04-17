use borsh_1_6::{self as borsh, BorshDeserialize, BorshSerialize};

#[derive(BorshSerialize, BorshDeserialize)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UndelegateArgs {
    pub pda_seeds: Vec<Vec<u8>>,
}
