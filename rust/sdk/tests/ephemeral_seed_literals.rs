//! Compile the public macro with heterogeneous PDA seed literals.
#![cfg(feature = "anchor-modern")]
#![allow(unexpected_cfgs)]

extern crate anchor_lang_current as anchor_lang;

use anchor_lang::prelude::*;
use ephemeral_rollups_sdk::anchor::ephemeral_accounts;

declare_id!("11111111111111111111111111111111");

#[ephemeral_accounts]
#[derive(Accounts)]
pub struct WalletSponsor<'info> {
    #[account(mut, sponsor)]
    pub payer: Signer<'info>,
    /// CHECK: PDA constrained by seeds.
    #[account(mut, eph, seeds = [b"game", b"x"], bump)]
    pub unequal: UncheckedAccount<'info>,
    /// CHECK: Equal-length literals are a control for existing behavior.
    #[account(mut, eph, seeds = [b"same", b"size"], bump)]
    pub equal: UncheckedAccount<'info>,
    /// CHECK: Explicit slices remain supported.
    #[account(mut, eph, seeds = [b"slice", b"x".as_ref()], bump)]
    pub sliced: UncheckedAccount<'info>,
    /// CHECK: Mixed literals and account-derived seeds retain key lifetimes.
    #[account(mut, eph, seeds = [b"dynamic", payer.key().as_ref(), b"x"], bump)]
    pub dynamic: UncheckedAccount<'info>,
}

#[ephemeral_accounts]
#[derive(Accounts)]
pub struct PdaSponsor<'info> {
    /// CHECK: Sponsor PDA constrained by seeds.
    #[account(mut, sponsor, seeds = [b"treasury", b"x"], bump)]
    pub treasury: UncheckedAccount<'info>,
    /// CHECK: Ephemeral PDA constrained by seeds.
    #[account(mut, eph, seeds = [b"game", treasury.key().as_ref()], bump)]
    pub game: UncheckedAccount<'info>,
}

// Typecheck the generated methods without invoking a CPI.
fn wallet_methods(accounts: &WalletSponsor<'_>) -> Result<()> {
    accounts.create_ephemeral_unequal(32)?;
    accounts.init_if_needed_ephemeral_unequal(32)?;
    accounts.resize_ephemeral_unequal(64)?;
    accounts.close_ephemeral_unequal()?;
    accounts.create_ephemeral_equal(32)?;
    accounts.create_ephemeral_sliced(32)?;
    accounts.create_ephemeral_dynamic(32)
}

fn pda_methods(accounts: &PdaSponsor<'_>) -> Result<()> {
    accounts.create_ephemeral_game(32)?;
    accounts.init_if_needed_ephemeral_game(32)?;
    accounts.resize_ephemeral_game(64)?;
    accounts.close_ephemeral_game()
}

#[test]
fn wallet_sponsor_methods_compile() {
    let _ = wallet_methods;
}

#[test]
fn pda_sponsor_methods_compile() {
    let _ = pda_methods;
}
