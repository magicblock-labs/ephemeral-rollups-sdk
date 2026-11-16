# Ephemeral Rollups SDK (Rust)

CPIs, intent builders, and Anchor macros for delegation, commits, undelegation,
and Magic accounts.

```toml
[dependencies]
ephemeral-rollups-sdk = { version = "0.20.0", features = ["anchor"] }
```

## SDK versioning for Validator v1 and v2

The Rust and TypeScript SDKs share a version number. The `0.17.x` line targets
MagicBlock Validator v1. The `0.18.x` and `0.19.x` slots are reserved for
future v1-compatible releases if breaking SDK changes are needed.

The `0.20.x` line is separate and targets Validator v2. Version `1.0.0` is
reserved for the planned major SDK redesign.

## Features

| Integration | Feature |
|-------------|---------|
| Anchor 1.2.2+ | `anchor` |
| Modular Solana crates | `modular-sdk` |
| Permissions, tokens, randomness, scheduled tasks | `access-control`, `spl`, `vrf`, `crank`, respectively |

The SDK targets Solana 3.0. Anchor integration requires Anchor 1.2.2 or later.
`spl` also enables `encryption` and `instruction`.

## Anchor macros

Import macros from `ephemeral_rollups_sdk::anchor`:

- [`#[ephemeral]`](../ephemeral/README.md): add the undelegation handler to a program module.
- [`#[delegate]`](../delegate/README.md): generate delegation accounts and helpers.
- [`#[commit]`](../commit-attribute/README.md): add Magic Program accounts to an accounts struct.
- [`#[action]`](../action-attribute/README.md): add callback escrow accounts.
- [`#[magic_accounts]`](../ephemeral-accounts-attribute/README.md): create, resize, and close Magic accounts.

Delegation requires untyped accounts: Anchor must not serialize a delegated
account after its ownership changes. See the delegation guide before updating
and delegating an account in the same instruction.

[Integration guides](https://docs.magicblock.gg/) ·
[API reference](https://docs.rs/ephemeral-rollups-sdk)
