//! Minimal Anchor counter demonstrating the full Ephemeral Rollups lifecycle:
//! initialize -> increment (base) -> delegate -> increment (ER) -> commit ->
//! commit_and_undelegate.
//!
//! The `#[ephemeral]` macro injects the `process_undelegation` handler that the
//! delegation program calls back into when an account is undelegated. The
//! `#[delegate]` macro injects the delegation buffer/record/metadata accounts and a
//! `delegate_<field>` helper. The `#[commit]` macro injects the `magic_program` and
//! `magic_context` accounts used to schedule commits.

use anchor_lang::prelude::*;
use ephemeral_rollups_sdk::anchor::{commit, delegate, ephemeral};
use ephemeral_rollups_sdk::cpi::DelegateConfig;
use ephemeral_rollups_sdk::ephem::{commit_accounts, commit_and_undelegate_accounts};

declare_id!("BwhHSZ7Zq8obyZTLESJbBxabHUoXGpHrmm2AJzNMzimL");

/// Seed for the single counter PDA owned by this program.
pub const COUNTER_SEED: &[u8] = b"counter";

#[ephemeral]
#[program]
pub mod counter_anchor {
    use super::*;

    /// Create the counter PDA on the base layer.
    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        ctx.accounts.counter.count = 0;
        Ok(())
    }

    /// Increment the counter. Runs on the base layer before delegation and on the
    /// ephemeral rollup once the account is delegated.
    pub fn increment(ctx: Context<Increment>) -> Result<()> {
        ctx.accounts.counter.count = ctx.accounts.counter.count.wrapping_add(1);
        Ok(())
    }

    /// Delegate the counter PDA to the delegation program so it can be operated on
    /// the ephemeral rollup.
    ///
    /// `validator` is the identity of the ephemeral validator that may operate the
    /// account. Pass the identity of the ER you intend to run on; on a network with
    /// a router you would typically resolve it dynamically.
    pub fn delegate(ctx: Context<DelegateInput>, validator: Pubkey) -> Result<()> {
        let payer_key = ctx.accounts.payer.key();
        ctx.accounts.delegate_counter(
            &ctx.accounts.payer,
            &[COUNTER_SEED, payer_key.as_ref()],
            DelegateConfig {
                commit_frequency_ms: 30_000,
                validator: Some(validator),
            },
        )?;
        Ok(())
    }

    /// Schedule a commit of the counter state from the ER back to the base layer.
    pub fn commit(ctx: Context<CommitCounter>) -> Result<()> {
        commit_accounts(
            &ctx.accounts.payer,
            vec![&ctx.accounts.counter.to_account_info()],
            &ctx.accounts.magic_context,
            &ctx.accounts.magic_program,
            None,
        )?;
        Ok(())
    }

    /// Commit the counter state and undelegate it back to the program.
    pub fn commit_and_undelegate(ctx: Context<CommitCounter>) -> Result<()> {
        commit_and_undelegate_accounts(
            &ctx.accounts.payer,
            vec![&ctx.accounts.counter.to_account_info()],
            &ctx.accounts.magic_context,
            &ctx.accounts.magic_program,
            None,
        )?;
        Ok(())
    }
}

#[account]
pub struct Counter {
    pub count: u64,
}

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,
    #[account(
        init,
        payer = payer,
        space = 8 + 8,
        seeds = [COUNTER_SEED, payer.key().as_ref()],
        bump
    )]
    pub counter: Account<'info, Counter>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct Increment<'info> {
    pub payer: Signer<'info>,
    #[account(mut, seeds = [COUNTER_SEED, payer.key().as_ref()], bump)]
    pub counter: Account<'info, Counter>,
}

/// Accounts for delegation. The `del` marker on `counter` makes the `#[delegate]`
/// macro inject the buffer/record/metadata PDAs and the `delegate_counter` helper.
#[delegate]
#[derive(Accounts)]
pub struct DelegateInput<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,
    /// CHECK: the counter PDA to delegate
    #[account(mut, del)]
    pub counter: AccountInfo<'info>,
}

/// Accounts for committing. The `#[commit]` macro injects `magic_program` and
/// `magic_context`.
#[commit]
#[derive(Accounts)]
pub struct CommitCounter<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,
    #[account(mut, seeds = [COUNTER_SEED, payer.key().as_ref()], bump)]
    pub counter: Account<'info, Counter>,
}
