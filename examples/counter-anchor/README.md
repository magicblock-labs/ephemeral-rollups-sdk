# counter-anchor

A minimal **Anchor** program demonstrating the full Ephemeral Rollups lifecycle:

```text
initialize (base) → increment (base) → delegate → increment (ER) → commit → commit_and_undelegate
```

It exercises the SDK's headline anchor macros: `#[ephemeral]` (injects the
`process_undelegation` callback), `#[delegate]` (injects the delegation
buffer/record/metadata accounts and a `delegate_*` helper), and `#[commit]`
(injects the `magic_program` / `magic_context` accounts).

The counter PDA is seeded per payer (`["counter", payer]`) so each test run uses an
isolated account.

## Layout

| Path                   | What                                                                              |
| ---------------------- | --------------------------------------------------------------------------------- |
| `src/lib.rs`           | the program                                                                       |
| `program-keypair.json` | fixed program id (so `declare_id!` matches the deployed binary)                   |
| `tests/web3js.test.ts` | lifecycle test using `@solana/web3.js` + `@magicblock-labs/ephemeral-rollups-sdk` |
| `tests/kit.test.ts`    | lifecycle test using `@solana/kit` + `@magicblock-labs/ephemeral-rollups-kit`     |
| `tests/_shared.ts`     | framework-agnostic helpers (endpoints, discriminators, decoding)                  |

## Running locally

Prerequisites: Rust + Solana CLI (with `cargo build-sbf`), Node, and the validator
binaries:

```bash
yarn global add @magicblock-labs/ephemeral-validator@latest
```

Then, from the repo root:

```bash
# 1. build the program
cd examples/counter-anchor && cargo build-sbf && cd -

# 2. start the local stack (base + ER + query-filtering-service),
#    preloading the freshly built program onto the base layer
examples/scripts/start-validators.sh examples/counter-anchor/target/deploy

# 3. run both test suites
cd examples/counter-anchor && yarn install && yarn test && cd -

# 4. tear the stack down
examples/scripts/stop-validators.sh
```

Endpoints (defaults, overridable via env): base `http://127.0.0.1:8899`, rollup
(through the query-filtering-service) `http://127.0.0.1:2999`.

## Notes

- The local `ephemeral-validator` only operates accounts delegated to **its**
  identity, so `delegate` takes the target validator as an argument. The tests pass
  the well-known local dev identity (`ER_VALIDATOR_IDENTITY`).
- The query-filtering-service requires a JWT, obtained by signing a challenge with
  `getAuthToken` and appended to the RPC URL as `?token=...`.
- ER transactions skip preflight: the non-delegated fee payer trips the ER's
  preflight verification even though execution succeeds.
