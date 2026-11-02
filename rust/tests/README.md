# Rust tests

These files are registered as test targets in [sdk/Cargo.toml](../sdk/Cargo.toml),
not a standalone crate. Shared fixtures live in `common/`; crate-local unit tests
also live alongside their implementations.

Run from `rust/`:

```bash
cargo test -p ephemeral-rollups-sdk
cargo test -p ephemeral-rollups-sdk --test unit_test
cargo test -p ephemeral-rollups-sdk --features access-control,modular-sdk --test access_control_test
cargo test -p ephemeral-rollups-sdk --features spl,access-control,modular-sdk --test spl_test
```

Feature-gated targets are skipped unless their required features are enabled.
Do not use `--all-features`: the modern and compatibility Anchor features are
mutually exclusive. For the feature build matrix, run
`./scripts/test-combinations.sh --no-color` from `rust/sdk/`.

Some targets, including `macros_test`, contain placeholder tests; a passing suite
is not evidence of end-to-end delegation or validator behavior.
