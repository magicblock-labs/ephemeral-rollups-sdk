# ephemeral-rollups-sdk-attribute-ephemeral-accounts

Procedural macro for creating and managing ephemeral accounts in Ephemeral Rollups.

## Overview

Ephemeral accounts are zero-balance accounts that exist only in the Ephemeral Rollup. Rent is 109x cheaper than Solana (32 lamports/byte).

The `#[ephemeral_accounts]` macro simplifies working with ephemeral accounts by:
- Auto-generating `vault` and `magic_program` fields
- Generating helper methods for account lifecycle management
- Extracting seeds from Anchor's `#[account]` attributes

## Installation

```toml
[dependencies]
ephemeral-rollups-sdk = { version = "0.8", features = ["anchor"] }
```

## Account Markers

| Marker | Description |
|--------|-------------|
| `eph` | Marks a field as an ephemeral account |
| `sponsor` | Marks the account that pays rent |

**Note:** `eph` cannot be combined with Anchor's `init` or `init_if_needed`. Use the generated methods instead.

## Generated Methods

For each field marked with `eph`, the following methods are generated:

| Method | Description |
|--------|-------------|
| `create_ephemeral_<field>(data_len)` | Creates the ephemeral account |
| `init_if_needed_ephemeral_<field>(data_len)` | Creates only if account doesn't exist |
| `resize_ephemeral_<field>(new_data_len)` | Resizes the account |
| `close_ephemeral_<field>()` | Closes account, refunds rent to sponsor |

## Basic Usage (Wallet Sponsor)

```rust
use anchor_lang::prelude::*;
use ephemeral_rollups_sdk::anchor::ephemeral_accounts;

#[ephemeral_accounts]
#[derive(Accounts)]
pub struct CreateGame<'info> {
    #[account(mut, sponsor)]  // <-- sponsor pays rent
    pub payer: Signer<'info>,

    /// CHECK: Ephemeral account
    #[account(
        mut,
        eph,  // <-- ephemeral marker
        seeds = [b"game", payer.key().as_ref()],
        bump,
    )]
    pub game_state: AccountInfo<'info>,
}

// Create ephemeral account
pub fn create_game(ctx: Context<CreateGame>) -> Result<()> {
    ctx.accounts.create_ephemeral_game_state(1000)?;
    Ok(())
}

// Create only if doesn't exist (like Anchor's init_if_needed)
pub fn create_game_if_needed(ctx: Context<CreateGame>) -> Result<()> {
    ctx.accounts.init_if_needed_ephemeral_game_state(1000)?;
    Ok(())
}

// Resize existing account
pub fn resize_game(ctx: Context<CreateGame>) -> Result<()> {
    ctx.accounts.resize_ephemeral_game_state(2000)?;
    Ok(())
}

// Close and refund rent
pub fn close_game(ctx: Context<CreateGame>) -> Result<()> {
    ctx.accounts.close_ephemeral_game_state()?;
    Ok(())
}
```

**Auto-generated fields:**
- `vault` - ephemeral rent vault
- `magic_program` - magic program

## PDA Sponsor

When the sponsor is a PDA, specify seeds:

```rust
#[ephemeral_accounts]
#[derive(Accounts)]
pub struct CreateGameWithTreasury<'info> {
    /// CHECK: PDA treasury as sponsor
    #[account(
        mut,
        sponsor,
        seeds = [b"treasury"],
        bump,
    )]
    pub treasury: AccountInfo<'info>,

    /// CHECK: Ephemeral account
    #[account(
        mut,
        eph,
        seeds = [b"game", treasury.key().as_ref()],
        bump,
    )]
    pub game_state: AccountInfo<'info>,
}
```

## Oncurve Ephemeral Account

For ephemeral accounts backed by a keypair (must sign transaction):

```rust
#[ephemeral_accounts]
#[derive(Accounts)]
pub struct CreateWithKeypair<'info> {
    #[account(mut, sponsor)]
    pub payer: Signer<'info>,

    /// CHECK: Oncurve ephemeral - must sign the transaction
    #[account(mut, eph)]
    pub temp_account: Signer<'info>,  // No seeds needed
}
```

## Multiple Ephemeral Accounts

```rust
#[ephemeral_accounts]
#[derive(Accounts)]
pub struct MultiEphemeral<'info> {
    #[account(mut, sponsor)]
    pub payer: Signer<'info>,

    /// CHECK: First ephemeral account
    #[account(mut, eph, seeds = [b"player", payer.key().as_ref()], bump)]
    pub player_state: AccountInfo<'info>,

    /// CHECK: Second ephemeral account
    #[account(mut, eph, seeds = [b"game", payer.key().as_ref()], bump)]
    pub game_state: AccountInfo<'info>,
}

pub fn create_both(ctx: Context<MultiEphemeral>) -> Result<()> {
    ctx.accounts.init_if_needed_ephemeral_player_state(100)?;
    ctx.accounts.init_if_needed_ephemeral_game_state(200)?;
    Ok(())
}
```

## Signing Requirements

| Operation | Sponsor | Ephemeral |
|-----------|---------|-----------|
| Create | Must sign | Must sign (prevents squatting) |
| Resize | Must sign | Does not sign |
| Close | Must sign | Does not sign |

## Rent Calculation

```rust
use ephemeral_rollups_sdk::ephemeral_accounts::rent;

let cost = rent(1000);  // Cost for 1KB account
// cost = (1000 + 60) * 32 = 33,920 lamports
```

## Resources

- [Ephemeral Accounts Specification](https://docs.magicblock.gg/)
- [Ephemeral Rollups Documentation](https://docs.magicblock.gg/)
