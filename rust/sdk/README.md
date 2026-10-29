# Ephemeral Rollups SDK (Rust)

CPIs, intent builders, and Anchor macros for delegation, commits, undelegation,
and ephemeral accounts.

```toml
[dependencies]
ephemeral-rollups-sdk = { version = "0.17", features = ["anchor"] }
```

## Features

| Integration | Feature |
|-------------|---------|
| Anchor 1.0 | `anchor` (alias for `anchor-modern`) |
| Anchor >=0.28, <1.0 | `anchor-compat` (includes `backward-compat`) |
| Legacy Solana types without Anchor | `backward-compat` |
| Modular Solana crates | `modular-sdk` |
| Permissions, tokens, randomness, scheduled tasks | `access-control`, `spl`, `vrf`, `crank`, respectively |

Without compatibility flags, the SDK targets Solana 3.0. Do not combine `anchor`
with `anchor-compat` or `backward-compat`; these combinations fail at compile time.
`spl` also enables `encryption` and `instruction`.

## Anchor macros

Import macros from `ephemeral_rollups_sdk::anchor`:

- [`#[ephemeral]`](../ephemeral/README.md): add the undelegation handler to a program module.
- [`#[delegate]`](../delegate/README.md): generate delegation accounts and helpers.
- [`#[commit]`](../commit-attribute/README.md): add Magic Program accounts to an accounts struct.
- [`#[action]`](../action-attribute/README.md): add callback escrow accounts.
- [`#[ephemeral_accounts]`](../ephemeral-accounts-attribute/README.md): create, resize, and close ephemeral-only accounts.

Delegation requires untyped accounts: Anchor must not serialize a delegated
account after its ownership changes. See the delegation guide before updating
and delegating an account in the same instruction.

[Integration guides](https://docs.magicblock.gg/) ·
[API reference](https://docs.rs/ephemeral-rollups-sdk)
