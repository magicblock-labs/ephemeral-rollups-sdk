#[cfg(feature = "anchor-support")]
#[allow(unused_imports)]
use crate::compat::anchor_lang;

#[cfg(feature = "anchor-support")]
use crate::compat::anchor_lang::{AnchorDeserialize, AnchorSerialize};

//#[cfg(feature = "anchor-support")]
#[allow(unused_imports)]
use crate::compat::borsh;

#[cfg(not(feature = "anchor-support"))]
use crate::compat::borsh::{BorshDeserialize, BorshSerialize};

#[cfg_attr(feature = "anchor-support", derive(AnchorSerialize, AnchorDeserialize))]
#[cfg_attr(
    not(feature = "anchor-support"),
    derive(BorshSerialize, BorshDeserialize)
)]
#[cfg_attr(
    all(not(feature = "anchor-support"), not(feature = "backward-compat")),
    borsh(crate = "crate::compat::borsh")
)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UndelegateArgs {
    pub pda_seeds: Vec<Vec<u8>>,
}
