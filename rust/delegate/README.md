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

    #[account(mut, del, seeds = [b"player", payer.key().as_ref()], bump)]
    pub player: Account<'info, Player>,
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

`del` fields may use typed accounts such as `Account<T>` with Anchor 1.2.2 or
later. If delegation changes ownership during the instruction, Anchor permits
account exit when the data is unchanged and rejects pending data changes. To
update and delegate in one instruction, call `.exit(&crate::ID)` **before**
delegating so the updated data is written while the account is still owned by
your program.

```rust
#[account]
pub struct Player {
    pub score: u64,
}
```

[Integration guides](https://docs.magicblock.gg/)
