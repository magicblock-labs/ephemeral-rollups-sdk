# ⚡ Ephemeral Rollups SDK

Rust and TypeScript tools for delegating accounts, executing on Ephemeral Rollups,
and committing state back to Solana.

## Choose a package

| Package | Use |
|---------|-----|
| [ephemeral-rollups-sdk](rust/sdk/README.md) | Rust program integration, CPIs, and Anchor macros |
| [ephemeral-rollups-pinocchio](rust/pinocchio/README.md) | Pinocchio program integration |
| [magic-resolver](rust/resolver/README.md) | Rust client-side connection routing by delegation status |
| [@magicblock-labs/ephemeral-rollups-sdk](ts/web3js/README.md) | TypeScript clients using `@solana/web3.js` |
| [@magicblock-labs/ephemeral-rollups-kit](ts/kit/README.md) | TypeScript clients using `@solana/kit` |

For Rust, choose the [compatibility features](rust/sdk/README.md#features) before
integrating: `anchor` targets Anchor 1.0; older Anchor versions use `anchor-compat`.
Delegated accounts and [ephemeral-only accounts](rust/ephemeral-accounts-attribute/README.md)
have different lifecycles; ephemeral-only accounts do not persist on Solana.

[Integration guides](https://docs.magicblock.gg/) ·
[Example programs](https://github.com/magicblock-labs/magicblock-engine-examples)
