# TODO

## Reuse args types from magicblock-magic-program-api

Currently `intent_bundle.rs` defines its own args types (`ActionArgs`, `ShortAccountMeta`, `BaseActionArgs`, `CommitTypeArgs`, `UndelegateTypeArgs`, `CommitAndUndelegateArgs`, `MagicIntentBundleArgs`) with manual bincode serialization.

These duplicate the types in `magicblock-magic-program-api` crate.

### Action required

1. Add a feature flag to `magicblock-magic-program-api` (e.g., `pinocchio`) that:
   - Uses `pinocchio::pubkey::Pubkey` instead of `solana_program::pubkey::Pubkey`
   - Supports `no_std` with `alloc`
   - Either uses serde with `no_std` support or provides manual serialization

2. Once the feature is available, update `pinocchio/Cargo.toml`:
   ```toml
   [dependencies]
   magicblock-magic-program-api = { workspace = true, features = ["pinocchio"] }
   ```

3. Remove duplicated args types from `intent_bundle.rs` and import from `magicblock-magic-program-api`
