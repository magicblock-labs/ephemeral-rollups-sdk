# ephemeral-rollups-sdk-attribute-ephemeral-accounts

`#[ephemeral_accounts]` manages zero-balance accounts that exist only in an
Ephemeral Rollup. A sponsor pays rent into a separate vault.

Use the SDK's [Anchor feature](../sdk/README.md#features) matching your Anchor
version. Place the macro before `#[derive(Accounts)]`:

```rust
use anchor_lang::prelude::*;
use ephemeral_rollups_sdk::anchor::ephemeral_accounts;

#[ephemeral_accounts]
#[derive(Accounts)]
pub struct CreateGame<'info> {
    #[account(mut, sponsor)]
    pub payer: Signer<'info>,

    /// CHECK: PDA address is constrained; initialized through the generated helper.
    #[account(mut, eph, seeds = [b"game", payer.key().as_ref()], bump)]
    pub game_state: AccountInfo<'info>,
}

pub fn create_game(ctx: Context<CreateGame>) -> Result<()> {
    ctx.accounts.create_ephemeral_game_state(1000)?;
    Ok(())
}
```

The macro adds missing `vault` and `magic_program` fields and generates these
methods for each `eph` field:

| Method | Behavior |
|--------|----------|
| `create_ephemeral_<field>(data_len)` | Create with the requested data size |
| `init_if_needed_ephemeral_<field>(data_len)` | Create only when current data length is zero |
| `resize_ephemeral_<field>(new_data_len)` | Resize; sponsor pays or receives the rent difference |
| `close_ephemeral_<field>()` | Close and refund rent to the sponsor |

## Constraints that matter

- Exactly one `sponsor` is required when using `eph`; multiple ephemeral fields
  can share it. Do not combine `eph` with Anchor's `init` or `init_if_needed`.
- A wallet sponsor uses `Signer`. A PDA sponsor needs `seeds` and `bump`;
  the macro signs its CPI using those seeds.
- A keypair-backed ephemeral account uses `Signer` without seeds. For a PDA,
  supply seeds and a bump as above.
- Creation requires both sponsor and ephemeral account signatures. Resize and
  close require only the sponsor signature; PDA signatures are supplied via CPI.
- `init_if_needed_ephemeral_*` checks only data length, not ownership or the
  requested size. It does not resize or validate an existing nonempty account.

Use `ephemeral_rollups_sdk::ephemeral_accounts::rent(data_len)` to calculate rent,
including the 60-byte account overhead. The SDK formula is
`(data_len + 60) * 32` lamports: 1,000 data bytes cost 33,920 lamports.

[Integration guides](https://docs.magicblock.gg/)
