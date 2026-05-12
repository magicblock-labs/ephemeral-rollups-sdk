#[cfg(feature = "anchor")]
#[allow(unused_imports)]
use crate::compat::anchor_lang;
#[cfg(feature = "anchor")]
use crate::compat::anchor_lang::{AnchorDeserialize, AnchorSerialize};
#[cfg(feature = "anchor")]
#[allow(unused_imports)]
use crate::compat::borsh;

#[cfg(not(feature = "anchor"))]
use crate::compat::borsh::{BorshDeserialize, BorshSerialize};

#[cfg_attr(feature = "anchor", derive(AnchorSerialize, AnchorDeserialize))]
#[cfg_attr(not(feature = "anchor"), derive(BorshSerialize, BorshDeserialize))]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UndelegateArgs {
    pub pda_seeds: Vec<Vec<u8>>,
}
