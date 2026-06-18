//! Anchor example of the `#[ephemeral_accounts]` macro (gas-sponsored ephemeral
//! accounts created on the rollup).
//!
//! The macro funds the new `eph` account's rent from a `sponsor`. On the ER only
//! delegated accounts may be debited, so the sponsor here is a program-owned
//! `treasury` PDA that is first delegated to the ER (the `init_treasury` /
//! `delegate_treasury` instructions). `create_game` then runs on the ER with a normal
//! wallet as the (gasless) fee payer; the magic program debits the delegated treasury
//! for rent and creates `game_state`.
use anchor_lang::prelude::*;
use anchor_lang::system_program;
use ephemeral_rollups_sdk::anchor::{delegate, ephemeral_accounts};
use ephemeral_rollups_sdk::cpi::DelegateConfig;

declare_id!("CQeCqYqEWKFSm8GuEnGggSF4VBugU52DK9aR6q4cCiUf");

pub const TREASURY: &[u8] = b"treasury";
pub const GAME: &[u8] = b"game";

#[program]
pub mod ephemeral_accounts_anchor {
    use super::*;

    /// Create the sponsor treasury PDA on the base layer and fund it with `fund`
    /// lamports (used to sponsor ephemeral-account rent).
    pub fn init_treasury(ctx: Context<InitTreasury>, fund: u64) -> Result<()> {
        let cpi = system_program::Transfer {
            from: ctx.accounts.payer.to_account_info(),
            to: ctx.accounts.treasury.to_account_info(),
        };
        system_program::transfer(
            CpiContext::new(ctx.accounts.system_program.to_account_info(), cpi),
            fund,
        )?;
        Ok(())
    }

    /// Delegate the treasury PDA to the ephemeral validator so it can be debited there.
    pub fn delegate_treasury(ctx: Context<DelegateTreasury>, validator: Pubkey) -> Result<()> {
        let payer_key = ctx.accounts.payer.key();
        ctx.accounts.delegate_treasury(
            &ctx.accounts.payer,
            &[TREASURY, payer_key.as_ref()],
            DelegateConfig {
                commit_frequency_ms: 30_000,
                validator: Some(validator),
            },
        )?;
        Ok(())
    }

    /// Create the gas-sponsored ephemeral `game_state` account (size bytes) on the ER,
    /// funded from the delegated treasury.
    pub fn create_game(ctx: Context<CreateGame>, size: u32) -> Result<()> {
        ctx.accounts.create_ephemeral_game_state(size)?;
        Ok(())
    }
}

#[account]
pub struct Treasury {}

#[derive(Accounts)]
pub struct InitTreasury<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,
    #[account(init, payer = payer, space = 8, seeds = [TREASURY, payer.key().as_ref()], bump)]
    pub treasury: Account<'info, Treasury>,
    pub system_program: Program<'info, System>,
}

#[delegate]
#[derive(Accounts)]
pub struct DelegateTreasury<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,
    /// CHECK: the treasury PDA to delegate
    #[account(mut, del)]
    pub treasury: AccountInfo<'info>,
}

#[ephemeral_accounts]
#[derive(Accounts)]
pub struct CreateGame<'info> {
    /// CHECK: treasury authority, used only to derive the PDA seeds.
    pub authority: UncheckedAccount<'info>,
    /// CHECK: delegated sponsor treasury PDA.
    #[account(mut, sponsor, seeds = [TREASURY, authority.key().as_ref()], bump)]
    pub treasury: AccountInfo<'info>,
    /// CHECK: the gas-sponsored ephemeral account, created by the magic program.
    #[account(mut, eph, seeds = [GAME, authority.key().as_ref()], bump)]
    pub game_state: AccountInfo<'info>,
}
