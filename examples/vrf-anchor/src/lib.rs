//! Minimal Anchor VRF example: request verifiable randomness and consume it in a
//! callback fulfilled by the VRF oracle.
//!
//! - `#[vrf]` injects `program_identity` / `vrf_program` / `slot_hashes` and the
//!   `invoke_signed_vrf` helper used to issue a scoped randomness request.
//! - `#[vrf_callback]` injects the `vrf_program_identity` signer that proves the
//!   callback was issued by the VRF program for this program.
use anchor_lang::prelude::*;
use ephemeral_rollups_sdk::anchor::{vrf, vrf_callback};
use ephemeral_rollups_sdk::vrf::{
    self,
    instructions::{create_request_scoped_randomness_ix, RequestRandomnessParams},
    types::SerializableAccountMeta,
};

declare_id!("3YL1i4w5TD4bGuti3M1chRbtLeoPz2waVBphWXnpjyV5");

pub const RANDOM_SEED: &[u8] = b"random";

#[program]
pub mod vrf_anchor {
    use super::*;

    /// Create the per-payer randomness account.
    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        ctx.accounts.random.value = 0;
        Ok(())
    }

    /// Request randomness from the VRF oracle queue. The oracle fulfils it by calling
    /// `consume` back on this program.
    pub fn request(ctx: Context<RequestRandomness>, client_seed: u8) -> Result<()> {
        let ix = create_request_scoped_randomness_ix(RequestRandomnessParams {
            payer: ctx.accounts.payer.key(),
            oracle_queue: ctx.accounts.oracle_queue.key(),
            callback_program_id: ID,
            callback_discriminator: instruction::Consume::DISCRIMINATOR.to_vec(),
            caller_seed: [client_seed; 32],
            // The callback needs to write to the random account.
            accounts_metas: Some(vec![SerializableAccountMeta {
                pubkey: ctx.accounts.random.key(),
                is_signer: false,
                is_writable: true,
            }]),
            callback_args: None,
        });
        ctx.accounts
            .invoke_signed_vrf(&ctx.accounts.payer.to_account_info(), &ix)?;
        Ok(())
    }

    /// VRF callback: store a random number in 1..=100.
    pub fn consume(ctx: Context<ConsumeRandomness>, randomness: [u8; 32]) -> Result<()> {
        ctx.accounts.random.value = vrf::rnd::random_u8_with_range(&randomness, 1, 100);
        Ok(())
    }
}

#[account]
pub struct Random {
    pub value: u8,
}

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,
    #[account(
        init,
        payer = payer,
        space = 8 + 1,
        seeds = [RANDOM_SEED, payer.key().as_ref()],
        bump
    )]
    pub random: Account<'info, Random>,
    pub system_program: Program<'info, System>,
}

#[vrf]
#[derive(Accounts)]
pub struct RequestRandomness<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,
    #[account(mut, seeds = [RANDOM_SEED, payer.key().as_ref()], bump)]
    pub random: Account<'info, Random>,
    /// CHECK: the VRF oracle queue (validated against the known queues).
    #[account(
        mut,
        constraint = oracle_queue.key() == vrf::consts::DEFAULT_QUEUE
            || oracle_queue.key() == vrf::consts::DEFAULT_TEST_QUEUE
    )]
    pub oracle_queue: UncheckedAccount<'info>,
}

#[vrf_callback]
#[derive(Accounts)]
pub struct ConsumeRandomness<'info> {
    #[account(mut)]
    pub random: Account<'info, Random>,
}
