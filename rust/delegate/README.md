# ephemeral-rollups-sdk-attribute-delegate

Procedural macro attribute for delegating accounts in Ephemeral Rollups.

## Overview

The `#[delegate]` macro simplifies delegating accounts to Ephemeral Rollups by:
- Auto-generating buffer, delegation_record, and delegation_metadata fields
- Generating `delegate_<field>()` helper methods

## Installation

```toml
[dependencies]
ephemeral-rollups-sdk = { version = "0.8", features = ["anchor"] }
```

## Usage

```rust
use anchor_lang::prelude::*;
use ephemeral_rollups_sdk::anchor::delegate;

#[delegate]
#[derive(Accounts)]
pub struct DelegatePlayer<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,

    #[account(mut, del)]  // <-- delegation marker
    pub player: Account<'info, PlayerState>,
}

pub fn delegate_player(ctx: Context<DelegatePlayer>) -> Result<()> {
    let seeds: &[&[u8]] = &[b"player", ctx.accounts.payer.key.as_ref()];
    ctx.accounts.delegate_player(
        &ctx.accounts.payer,
        seeds,
        DelegateConfig::default(),
    )?;
    Ok(())
}
```

**Auto-generated fields:**
- `buffer_player`
- `delegation_record_player`
- `delegation_metadata_player`
- `owner_program`
- `delegation_program`
- `system_program`

**Auto-generated methods:**
- `delegate_player(payer, seeds, config)`

## Resources

- [Quickstart Guide](https://docs.magicblock.gg/pages/get-started/how-integrate-your-program/quickstart)
- [Ephemeral Rollups Documentation](https://docs.magicblock.gg/)
