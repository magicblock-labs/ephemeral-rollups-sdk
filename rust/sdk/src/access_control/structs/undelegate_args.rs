#[cfg(feature = "anchor")]
#[allow(unused_imports)]
use anchor_lang;

#[cfg(feature = "anchor")]
use anchor_lang::{AnchorDeserialize, AnchorSerialize};

//#[cfg(feature = "anchor")]
#[allow(unused_imports)]
use borsh;

#[cfg(not(feature = "anchor"))]
use borsh::{BorshDeserialize, BorshSerialize};

#[cfg_attr(feature = "anchor", derive(AnchorSerialize, AnchorDeserialize))]
#[cfg_attr(not(feature = "anchor"), derive(BorshSerialize, BorshDeserialize))]
#[cfg_attr(not(feature = "anchor"), borsh(crate = "borsh"))]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UndelegateArgs {
    pub pda_seeds: Vec<Vec<u8>>,
}
