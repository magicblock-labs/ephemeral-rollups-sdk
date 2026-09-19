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

    /// CHECK: delegated PDA, validated by seeds
    #[account(mut, del, seeds = [b"player", payer.key().as_ref()], bump)]  // <-- delegation marker
    pub player: UncheckedAccount<'info>,
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

`del` fields must be `UncheckedAccount` or `AccountInfo`; the macro rejects typed accounts
such as `Account<T>` at compile time. Anchor re-serializes typed accounts after the handler
returns, when the account is already owned by the delegation program, and that write fails
under direct mapping (SIMD-0460). To update the account in the same instruction, load it into
a local typed wrapper, mutate it, and call `.exit(&crate::ID)` on it before delegating.

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
