//! Compile the public macro with heterogeneous PDA seed literals.
#![cfg(feature = "anchor-modern")]
#![allow(unexpected_cfgs)]
#![allow(deprecated)]

extern crate anchor_lang_current as anchor_lang;

use anchor_lang::prelude::*;
use ephemeral_rollups_sdk::anchor::{ephemeral_accounts, magic_accounts};

declare_id!("11111111111111111111111111111111");

#[magic_accounts]
#[derive(Accounts)]
pub struct WalletSponsor<'info> {
    #[account(mut, sponsor)]
    pub payer: Signer<'info>,
    /// CHECK: PDA constrained by seeds.
    #[account(mut, magic, seeds = [b"game", b"x"], bump)]
    pub unequal: UncheckedAccount<'info>,
    /// CHECK: Equal-length literals are a control for existing behavior.
    #[account(mut, magic, seeds = [b"same", b"size"], bump)]
    pub equal: UncheckedAccount<'info>,
    /// CHECK: Explicit slices remain supported.
    #[account(mut, magic, seeds = [b"slice", b"x".as_ref()], bump)]
    pub sliced: UncheckedAccount<'info>,
    /// CHECK: Mixed literals and account-derived seeds retain key lifetimes.
    #[account(mut, magic, seeds = [b"dynamic", payer.key().as_ref(), b"x"], bump)]
    pub dynamic: UncheckedAccount<'info>,
}

#[magic_accounts]
#[derive(Accounts)]
pub struct PdaSponsor<'info> {
    /// CHECK: Sponsor PDA constrained by seeds.
    #[account(mut, sponsor, seeds = [b"treasury", b"x"], bump)]
    pub treasury: UncheckedAccount<'info>,
    /// CHECK: Magic account PDA constrained by seeds.
    #[account(mut, magic, seeds = [b"game", treasury.key().as_ref()], bump)]
    pub game: UncheckedAccount<'info>,
}

#[ephemeral_accounts]
#[derive(Accounts)]
pub struct LegacyAccountHelpers<'info> {
    #[account(mut, sponsor)]
    pub payer: Signer<'info>,
    /// CHECK: Existing `eph` syntax remains supported.
    #[account(mut, eph)]
    pub state: UncheckedAccount<'info>,
}

// Typecheck the generated methods without invoking a CPI.
fn wallet_methods(accounts: &WalletSponsor<'_>) -> Result<()> {
    accounts.create_magic_unequal(32)?;
    accounts.init_if_needed_magic_unequal(32)?;
    accounts.resize_magic_unequal(64)?;
    accounts.close_magic_unequal()?;
    accounts.create_magic_equal(32)?;
    accounts.create_magic_sliced(32)?;
    accounts.create_magic_dynamic(32)
}

fn pda_methods(accounts: &PdaSponsor<'_>) -> Result<()> {
    accounts.create_magic_game(32)?;
    accounts.init_if_needed_magic_game(32)?;
    accounts.resize_magic_game(64)?;
    accounts.close_magic_game()
}

fn legacy_methods(accounts: &LegacyAccountHelpers<'_>) -> Result<()> {
    accounts.create_ephemeral_state(32)?;
    accounts.init_if_needed_ephemeral_state(32)?;
    accounts.resize_ephemeral_state(64)?;
    accounts.close_ephemeral_state()
}

#[test]
fn wallet_sponsor_methods_compile() {
    let _ = wallet_methods;
}

#[test]
fn pda_sponsor_methods_compile() {
    let _ = pda_methods;
}

#[test]
fn legacy_api_paths_compile() {
    let _: Option<ephemeral_rollups_sdk::ephemeral_accounts::EphemeralAccount<'static, 'static>> =
        None;
    let _ = ephemeral_rollups_sdk::consts::EPHEMERAL_VAULT_ID;
    assert_eq!(
        ephemeral_rollups_sdk::consts::EPHEMERAL_VAULT_ID,
        ephemeral_rollups_sdk::consts::MAGIC_ACCOUNT_VAULT_ID
    );
    let _ = legacy_methods;
}
