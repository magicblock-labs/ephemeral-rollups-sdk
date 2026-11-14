//! Compile-time coverage for typed accounts passed to the `#[delegate]` macro.
#![allow(unexpected_cfgs)]

use anchor_lang::prelude::*;
use ephemeral_rollups_sdk::{anchor::delegate, cpi::DelegateConfig};

declare_id!("11111111111111111111111111111111");

#[account]
pub struct Player {
    pub score: u64,
}

#[delegate]
#[derive(Accounts)]
pub struct DelegatePlayer<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,
    #[account(mut, del, seeds = [b"player", payer.key().as_ref()], bump)]
    pub player: Account<'info, Player>,
}

fn verify_typed_delegate(ctx: &DelegatePlayer<'_>) {
    let _: &Account<'_, Player> = &ctx.player;
    let _ = ctx.delegate_player(
        &ctx.payer,
        &[b"player", ctx.payer.key().as_ref()],
        DelegateConfig::default(),
    );
}

#[test]
fn typed_delegate_account_compiles() {
    let _ = verify_typed_delegate as fn(&DelegatePlayer<'_>);
}
