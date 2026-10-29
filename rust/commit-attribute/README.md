# ephemeral-rollups-sdk-attribute-commit

`#[commit]` adds missing `magic_program` and writable `magic_context` fields to an
Anchor accounts struct. Import it from `ephemeral_rollups_sdk::anchor` and place
it before `#[derive(Accounts)]`.

The attribute only supplies accounts; the handler must still invoke the commit
or commit-and-undelegate operation.

See [SDK feature selection](../sdk/README.md#features) and the
[integration guides](https://docs.magicblock.gg/).
