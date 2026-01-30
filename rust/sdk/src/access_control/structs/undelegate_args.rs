#[cfg(feature = "anchor")]
use anchor_lang::prelude::*;

#[cfg(not(feature = "anchor"))]
use borsh::{BorshDeserialize, BorshSerialize};

#[cfg_attr(feature = "anchor", derive(AnchorSerialize, AnchorDeserialize))]
#[cfg_attr(not(feature = "anchor"), derive(BorshSerialize, BorshDeserialize))]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UndelegateArgs {
    pub pda_seeds: Vec<Vec<u8>>,
}
