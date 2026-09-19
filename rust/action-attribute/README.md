# ephemeral-rollups-sdk-attribute-action

`#[action]` adds missing `escrow_auth` and `escrow` fields to an Anchor accounts
struct for action callbacks. Import it from `ephemeral_rollups_sdk::anchor` and
place it before `#[derive(Accounts)]`.

Both generated fields are `UncheckedAccount`: the macro adds no signer or PDA
constraints. It does not authenticate a callback or schedule an action for you.

See [SDK feature selection](../sdk/README.md#features) and the
[integration guides](https://docs.magicblock.gg/).
