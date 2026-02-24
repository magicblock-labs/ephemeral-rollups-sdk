//! Compile-time tests for the `#[ephemeral_accounts]` macro.
//!
//! These tests verify that:
//! 1. The macro generates the expected methods
//! 2. Auto-injected fields (vault, magic_program) exist
//! 3. Method signatures are correct
//!
//! If any method or field is missing, the code won't compile.

#![allow(unexpected_cfgs)]

use anchor_lang::prelude::*;
use ephemeral_rollups_sdk::anchor::ephemeral_accounts;

declare_id!("11111111111111111111111111111111");

// ============ Test Structs ============

/// Scenario 1: Wallet sponsor + PDA ephemeral
#[ephemeral_accounts]
#[derive(Accounts)]
pub struct CreateGameWalletSponsor<'info> {
    #[account(mut, sponsor)]
    pub payer: Signer<'info>,

    #[account(mut, eph, seeds = [b"game", payer.key().as_ref()], bump)]
    pub game_state: AccountInfo<'info>,
}

/// Scenario 2: PDA sponsor + PDA ephemeral
#[ephemeral_accounts]
#[derive(Accounts)]
pub struct CreateGamePdaSponsor<'info> {
    #[account(mut, sponsor, seeds = [b"treasury"], bump)]
    pub treasury: AccountInfo<'info>,

    #[account(mut, eph, seeds = [b"game", treasury.key().as_ref()], bump)]
    pub game_state: AccountInfo<'info>,
}

/// Scenario 3: Wallet sponsor + oncurve ephemeral
#[ephemeral_accounts]
#[derive(Accounts)]
pub struct CreateTempOncurve<'info> {
    #[account(mut, sponsor)]
    pub payer: Signer<'info>,

    #[account(mut, eph)]
    pub temp_account: Signer<'info>,
}

/// Scenario 4: Multiple ephemeral accounts
#[ephemeral_accounts]
#[derive(Accounts)]
pub struct MultiEphemeral<'info> {
    #[account(mut, sponsor)]
    pub payer: Signer<'info>,

    #[account(mut, eph, seeds = [b"player", payer.key().as_ref()], bump)]
    pub player_state: AccountInfo<'info>,

    #[account(mut, eph, seeds = [b"game", payer.key().as_ref()], bump)]
    pub game_state: AccountInfo<'info>,
}

// ============ Compile-Time Verification Functions ============

/// Verify CreateGameWalletSponsor has all expected methods and fields
fn verify_wallet_sponsor(ctx: &CreateGameWalletSponsor<'_>) {
    // User fields exist
    let _: &Signer<'_> = &ctx.payer;
    let _: &AccountInfo<'_> = &ctx.game_state;

    // Auto-injected fields exist
    let _: &AccountInfo<'_> = &ctx.vault;
    let _: &Program<'_, ephemeral_rollups_sdk::anchor::MagicProgram> = &ctx.magic_program;

    // Methods exist with correct signatures
    let _: Result<()> = ctx.create_ephemeral_game_state(1000u32);
    let _: Result<()> = ctx.init_if_needed_ephemeral_game_state(1000u32);
    let _: Result<()> = ctx.resize_ephemeral_game_state(2000u32);
    let _: Result<()> = ctx.close_ephemeral_game_state();
}

/// Verify CreateGamePdaSponsor has all expected methods and fields
fn verify_pda_sponsor(ctx: &CreateGamePdaSponsor<'_>) {
    // User fields exist
    let _: &AccountInfo<'_> = &ctx.treasury;
    let _: &AccountInfo<'_> = &ctx.game_state;

    // Auto-injected fields exist
    let _: &AccountInfo<'_> = &ctx.vault;
    let _: &Program<'_, ephemeral_rollups_sdk::anchor::MagicProgram> = &ctx.magic_program;

    // Methods exist with correct signatures
    let _: Result<()> = ctx.create_ephemeral_game_state(1000u32);
    let _: Result<()> = ctx.init_if_needed_ephemeral_game_state(1000u32);
    let _: Result<()> = ctx.resize_ephemeral_game_state(2000u32);
    let _: Result<()> = ctx.close_ephemeral_game_state();
}

/// Verify CreateTempOncurve has all expected methods and fields
fn verify_oncurve(ctx: &CreateTempOncurve<'_>) {
    // User fields exist
    let _: &Signer<'_> = &ctx.payer;
    let _: &Signer<'_> = &ctx.temp_account;

    // Auto-injected fields exist
    let _: &AccountInfo<'_> = &ctx.vault;
    let _: &Program<'_, ephemeral_rollups_sdk::anchor::MagicProgram> = &ctx.magic_program;

    // Methods exist with correct signatures (named after temp_account field)
    let _: Result<()> = ctx.create_ephemeral_temp_account(1000u32);
    let _: Result<()> = ctx.init_if_needed_ephemeral_temp_account(1000u32);
    let _: Result<()> = ctx.resize_ephemeral_temp_account(2000u32);
    let _: Result<()> = ctx.close_ephemeral_temp_account();
}

/// Verify MultiEphemeral has all expected methods and fields
fn verify_multi(ctx: &MultiEphemeral<'_>) {
    // User fields exist
    let _: &Signer<'_> = &ctx.payer;
    let _: &AccountInfo<'_> = &ctx.player_state;
    let _: &AccountInfo<'_> = &ctx.game_state;

    // Auto-injected fields exist
    let _: &AccountInfo<'_> = &ctx.vault;
    let _: &Program<'_, ephemeral_rollups_sdk::anchor::MagicProgram> = &ctx.magic_program;

    // Methods for player_state field
    let _: Result<()> = ctx.create_ephemeral_player_state(100u32);
    let _: Result<()> = ctx.init_if_needed_ephemeral_player_state(100u32);
    let _: Result<()> = ctx.resize_ephemeral_player_state(200u32);
    let _: Result<()> = ctx.close_ephemeral_player_state();

    // Methods for game_state field
    let _: Result<()> = ctx.create_ephemeral_game_state(1000u32);
    let _: Result<()> = ctx.init_if_needed_ephemeral_game_state(1000u32);
    let _: Result<()> = ctx.resize_ephemeral_game_state(2000u32);
    let _: Result<()> = ctx.close_ephemeral_game_state();
}

// ============ Test ============

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compile_time_checks() {
        // Force compilation of all verify functions by taking function pointers
        let _ = verify_wallet_sponsor as fn(&CreateGameWalletSponsor<'_>);
        let _ = verify_pda_sponsor as fn(&CreateGamePdaSponsor<'_>);
        let _ = verify_oncurve as fn(&CreateTempOncurve<'_>);
        let _ = verify_multi as fn(&MultiEphemeral<'_>);
    }
}
