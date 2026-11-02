# ephemeral-rollups-pinocchio

Pinocchio helpers for delegation, intent bundles, ephemeral accounts, permissions,
tokens, scheduled tasks, and VRF.

```toml
[dependencies]
ephemeral-rollups-pinocchio = "0.17"
```

Targets Pinocchio 0.11 and `solana-address` 2.x. Enable `delegation-actions` for
delegation action support. Intent bundles are available without a feature flag;
`intent-bundle` is a compatibility no-op.

## Supported intent operations

Intent bundles support commits, commit-and-undelegate operations, and standalone
base-layer actions through `MagicIntentBundleBuilder`.

`commit_finalize` and `commit_finalize_and_undelegate` are unsupported by this
crate and cannot be requested through its public builders. Their private
serializer fields are reserved for wire-format compatibility and always encode
as `None`; they contribute no actions or callbacks. Keep these reserved fields
when working with the wire format.

Finalize support requires a future public API and compatible protocol payloads.
No release version or date is promised for these operations.

[Integration guides](https://docs.magicblock.gg/) ·
[API reference](https://docs.rs/ephemeral-rollups-pinocchio)
