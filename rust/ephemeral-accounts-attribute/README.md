# ephemeral-rollups-sdk-attribute-ephemeral-accounts

This crate provides the Anchor `#[magic_accounts]` macro for zero-balance Magic
accounts. A sponsor pays rent into a separate vault. The published crate name is
retained for dependency compatibility.

Use the SDK's [Anchor feature](../sdk/README.md#features) matching your Anchor
version. Place the macro before `#[derive(Accounts)]`:

```rust
use anchor_lang::prelude::*;
use ephemeral_rollups_sdk::anchor::magic_accounts;

#[magic_accounts]
#[derive(Accounts)]
pub struct CreateGame<'info> {
    #[account(mut, sponsor)]
    pub payer: Signer<'info>,

    /// CHECK: PDA address is constrained; initialized through the generated helper.
    #[account(mut, magic, seeds = [b"game", payer.key().as_ref()], bump)]
    pub game_state: UncheckedAccount<'info>,
}

pub fn create_game(ctx: Context<CreateGame>) -> Result<()> {
    ctx.accounts.create_magic_game_state(1000)?;
    Ok(())
}
```

The macro adds missing `vault` and `magic_program` fields and generates these
methods for each `magic` field:

| Method | Behavior |
|--------|----------|
| `create_magic_<field>(data_len)` | Create with the requested data size |
| `init_if_needed_magic_<field>(data_len)` | Create only when current data length is zero |
| `resize_magic_<field>(new_data_len)` | Resize; sponsor pays or receives the rent difference |
| `close_magic_<field>()` | Close and refund rent to the sponsor |

## Constraints that matter

- Exactly one `sponsor` is required when using `magic`; multiple Magic account
  fields can share it. Do not combine `magic` with Anchor's `init` or
  `init_if_needed`.
- A wallet sponsor uses `Signer`. A PDA sponsor needs `seeds` and `bump`;
  the macro signs its CPI using those seeds.
- A keypair-backed Magic account uses `Signer` without seeds. For a PDA,
  supply seeds and a bump as above.
- Creation requires both sponsor and Magic account signatures. Resize and
  close require only the sponsor signature; PDA signatures are supplied via CPI.
- `init_if_needed_magic_*` checks only data length, not ownership or the
  requested size. It does not resize or validate an existing nonempty account.

Use `ephemeral_rollups_sdk::magic_accounts::rent(data_len)` to calculate rent,
including the 60-byte account overhead. The SDK formula is
`(data_len + 60) * 32` lamports: 1,000 data bytes cost 33,920 lamports.

## Backward compatibility

The deprecated `#[ephemeral_accounts]` macro, `eph` field marker, and generated
`*_ephemeral_*` helper names remain available for existing programs.

[Integration guides](https://docs.magicblock.gg/)
