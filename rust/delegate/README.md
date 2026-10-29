# ephemeral-rollups-sdk-attribute-delegate

`#[delegate]` generates delegation accounts and a `delegate_<field>(payer, seeds,
config)` helper for each field marked `del`. Use the SDK's
[Anchor feature](../sdk/README.md#features) matching your Anchor version.

```rust
use anchor_lang::prelude::*;
use ephemeral_rollups_sdk::{anchor::delegate, cpi::DelegateConfig};

#[delegate]
#[derive(Accounts)]
pub struct DelegatePlayer<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,

    /// CHECK: PDA address is constrained; left untyped because delegation changes ownership.
    #[account(mut, del, seeds = [b"player", payer.key().as_ref()], bump)]
    pub player: UncheckedAccount<'info>,
}

pub fn delegate_player(ctx: Context<DelegatePlayer>) -> Result<()> {
    ctx.accounts.delegate_player(
        &ctx.accounts.payer,
        &[b"player", ctx.accounts.payer.key.as_ref()],
        DelegateConfig::default(),
    )?;
    Ok(())
}
```

Pass PDA seeds **without the bump**; the CPI helper derives it. The macro adds
`buffer_player`, `delegation_record_player`, `delegation_metadata_player`, plus
shared `owner_program`, `delegation_program`, and `system_program` fields.

`del` fields must be `UncheckedAccount` or `AccountInfo`; typed accounts such as
`Account<T>` are rejected. Anchor serializes typed accounts after the handler,
when delegation has already changed ownership, causing writes to fail under
direct mapping. To update and delegate in one instruction, load a local typed
wrapper, mutate it, and call `.exit(&crate::ID)` **before** delegating.

[Integration guides](https://docs.magicblock.gg/)
