# intent-bundle-pinocchio

The same Ephemeral Rollups counter lifecycle as `counter-pinocchio`, but the commit and
commit-and-undelegate steps are issued through the **intent bundle** builder
(`MagicIntentBundleBuilder`) rather than the standalone `commit_accounts` helpers.

A magic intent bundle can aggregate commits, undelegations and post-commit/base-layer
actions into a single magic-program instruction. This example uses the minimal
single-commit and single-commit-and-undelegate forms:

```rust
MagicIntentBundleBuilder::new(payer, magic_context, magic_program)
    .commit(&[counter])               // or .commit_and_undelegate(&[counter])
    .build_and_invoke(&mut data_buf)?;
```

| Path                   | What                                                              |
| ---------------------- | ----------------------------------------------------------------- |
| `src/lib.rs`           | the program (tag-dispatched; commit paths use the bundle builder) |
| `tests/web3js.test.ts` | `@solana/web3.js` + `@magicblock-labs/ephemeral-rollups-sdk`      |
| `tests/kit.test.ts`    | `@solana/kit` + `@magicblock-labs/ephemeral-rollups-kit`          |

## Running

```bash
cd examples/intent-bundle-pinocchio && cargo build-sbf && cd -
examples/scripts/start-validators.sh examples/intent-bundle-pinocchio/target/deploy
cd examples/intent-bundle-pinocchio && yarn install && yarn test && cd -
examples/scripts/stop-validators.sh
```
