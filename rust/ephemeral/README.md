# ephemeral-rollups-sdk-attribute-ephemeral

`#[ephemeral]` adds `process_undelegation` and its `InitializeAfterUndelegation`
accounts struct to an inline Anchor program module, along with intent-builder
trait imports. Import it from `ephemeral_rollups_sdk::anchor` and place it before
`#[program]`.

This is program-level undelegation support, not ephemeral account creation; use
[`#[ephemeral_accounts]`](../ephemeral-accounts-attribute/README.md) for that.

See [SDK feature selection](../sdk/README.md#features) and the
[integration guides](https://docs.magicblock.gg/).
