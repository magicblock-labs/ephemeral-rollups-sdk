# Ephemeral Rollups SDK — Examples

Ultra-minimal Solana programs demonstrating the Ephemeral Rollups SDK, in both
**Anchor** and **Pinocchio**, each tested from TypeScript with **both** SDK
flavours:

- `@magicblock-labs/ephemeral-rollups-sdk` (`@solana/web3.js`)
- `@magicblock-labs/ephemeral-rollups-kit` (`@solana/kit`)

Each example is self-contained (its own Cargo manifest and npm package, consuming
the in-repo crates/SDKs via path/`file:` deps) and is exercised end-to-end against a
real local validator stack. CI runs **one runner per example**.

## The local validator stack

`npm install -g @magicblock-labs/ephemeral-validator@latest` provides the binaries.
`examples/scripts/start-validators.sh [PROGRAMS_DIR]` boots:

| Service | Role | RPC / WS |
|---|---|---|
| `mb-test-validator` | base L1 (delegation, magic, VRF, ACL, SPL programs preloaded) | 8899 / 8900 |
| `ephemeral-validator` | the rollup (`--remotes` → base) | 7799 / 7800 |
| `query-filtering-service` | security proxy in front of the ER; the client's rollup endpoint | 2999 / 3000 |

`start-validators.sh` optionally preloads SBF programs from a directory (e.g. an
example's `target/deploy`). `stop-validators.sh` tears the stack down.

Tests talk to the base directly (`8899`) and to the rollup through the
query-filtering-service (`2999`). The router requires a JWT obtained with the SDK's
`getAuthToken` and appended as `?token=…`.

## Running an example

```bash
cd examples/<example> && cargo build-sbf && cd -
examples/scripts/start-validators.sh examples/<example>/target/deploy
cd examples/<example> && npm install && npm test && cd -
examples/scripts/stop-validators.sh
```

## Examples

| Example | Framework | Demonstrates |
|---|---|---|
| [`counter-anchor`](counter-anchor) | Anchor | delegate → run-on-ER → commit → commit-and-undelegate (`#[ephemeral]`/`#[delegate]`/`#[commit]`) |
| [`counter-pinocchio`](counter-pinocchio) | Pinocchio | the same lifecycle via the `ephemeral-rollups-pinocchio` helpers |
| [`access-control`](access-control) | client (SDK) | permission program: create / update / close a permission |
| [`vrf-anchor`](vrf-anchor) | Anchor | verifiable randomness: `#[vrf]` request + `#[vrf_callback]`, fulfilled by `vrf-oracle` |
| [`intent-bundle-pinocchio`](intent-bundle-pinocchio) | Pinocchio | commit / commit-and-undelegate via `MagicIntentBundleBuilder` |
| [`spl`](spl) | client (SDK) | ephemeral SPL token ATA: init vault, init ephemeral ATA, deposit tokens |

### Notes that apply to every example

- The local `ephemeral-validator` only operates accounts delegated to **its**
  identity, so `delegate` takes the target validator as an argument; the tests pass
  the well-known local dev identity (`ER_VALIDATOR_IDENTITY`).
- ER transactions skip preflight — the non-delegated fee payer trips the ER's
  preflight verification even though execution succeeds.
- PDAs are seeded per payer so the two test files (and repeated runs) stay isolated.
