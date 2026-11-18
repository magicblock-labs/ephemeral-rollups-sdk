# Ephemeral Rollups SDK (Rust)

CPIs, intent builders, and Anchor macros for delegation, commits, undelegation,
Magic Accounts, and ephemeral-only accounts.

```toml
[dependencies]
ephemeral-rollups-sdk = { version = "0.18.0", features = ["anchor"] }
```

## SDK versioning for Validator v1 and v2

The Rust and TypeScript SDKs share a version number.

| SDK version line | Validator support | Guidance |
| --- | --- | --- |
| `0.17.x` | Validator v1 | — |
| `0.18.x` | Validator v1 and v2 | Users are encouraged to migrate to `0.20.x` or later. |
| `0.19.x` | Reserved for possible v1-compatible releases with breaking SDK changes. | — |
| `0.20.x` and later | Validator v2 release line | Recommended for new integrations. |

Version `1.0.0` is reserved for the planned major SDK redesign. See the
[versioning guide](../../README.md#sdk-versioning-for-validator-v1-and-v2).

## Features

| Integration | Feature |
|-------------|---------|
| Anchor 1.0 | `anchor` (alias for `anchor-modern`) |
| Anchor >=0.28, <1.0 | `anchor-compat` (includes `backward-compat`) |
| Legacy Solana types without Anchor | `backward-compat` |
| Validator v2 Magic Account program | `magic-accounts` |
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
- `#[magic_accounts]`: generate helpers through Validator v2's dedicated Magic Account program.
  Requires `magic-accounts` and an Anchor feature.
- [`#[ephemeral_accounts]`](../ephemeral-accounts-attribute/README.md): create, resize, and close ephemeral-only accounts.

Delegation requires untyped accounts: Anchor must not serialize a delegated
account after its ownership changes. See the delegation guide before updating
and delegating an account in the same instruction.

[Integration guides](https://docs.magicblock.gg/) ·
[API reference](https://docs.rs/ephemeral-rollups-sdk)
