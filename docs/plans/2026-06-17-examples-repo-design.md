# Ephemeral Rollups SDK — Examples Collection Design

**Date:** 2026-06-17
**Branch:** `dode/examples`
**Status:** Implementation in progress

### Progress (updated 2026-06-17)

- ✅ Validator orchestration (`examples/scripts/`) — base + ER + query-filtering-service,
  validated end-to-end. Key findings baked in: the ER must be delegated to the local
  validator identity; the router needs a JWT (`getAuthToken` → `?token=`); ER txns skip
  preflight; daemons run in their own session (setsid/perl) and the ER start is retried.
- ✅ `counter-anchor` — full lifecycle, web3.js + kit tests green against the live stack.
- ✅ `counter-pinocchio` — same, via the `ephemeral-rollups-pinocchio` helpers.
- ✅ `.github/workflows/examples.yml` — one runner per example (matrix: both counters).
- ⏳ Remaining feature examples (actions, ephemeral-accounts, access-control, spl, vrf,
  intent-bundle) follow the proven counter pattern — see the matrix below.

## Goal

Create an `examples/` collection inside the `ephemeral-rollups-sdk` monorepo containing
ultra-minimal Solana programs that demonstrate **every** Ephemeral Rollups SDK feature,
written in both **Anchor** and **Pinocchio** (as separate examples). Each example ships
two TypeScript integration tests — one using `@magicblock-labs/ephemeral-rollups-sdk`
(`@solana/web3.js`) and one using `@magicblock-labs/ephemeral-rollups-kit` (`@solana/kit`)
— run with **vitest** against a real local validator stack. CI runs **one runner per
example**.

## Decisions (locked)

| Question | Decision |
|---|---|
| Location | `examples/` directory in this monorepo |
| Coverage milestone | All features (built incrementally, flagship-first) |
| Anchor vs Pinocchio layout | Separate examples (`*-anchor`, `*-pinocchio`) |
| Test runner | vitest, two files per example (`web3js.test.ts`, `kit.test.ts`) |

## Local validator stack

`npm install -g @magicblock-labs/ephemeral-validator@latest` (v0.12.3) provides binaries:
`mb-test-validator`, `ephemeral-validator`, `query-filtering-service`, `rpc-router`,
`vrf-oracle`.

`mb-test-validator` is `solana-test-validator` preloaded with every program/account an
example could need:

- Delegation program `DELeGGvXpWV2fqJUhqcF5ZSYMS4JTLjteaAMARRSaeSh`
- Magic program account `mAGicPQYBMvcYveUZA5F5UNNwyHvfYh5xkLS2Fr1mev` (+ magic context)
- VRF program `Vrf1RNUjXmQGjmQrQLvJHs9SNkvDJEsRVFPkfSQUwGz`
- ACL / permission program `ACLseoPoyC3cBqoUtkbjZ4aDrkurZW86v19pXz2XQnp1`
- Ephemeral SPL token program `SPLxh1LVZzEkX99H6rqYizhytLWPZVV296zyYDPagv2`
- noop + supporting accounts

This means **no example needs to clone/build the delegation program** — the base layer
ships it. Examples only build their own program.

### Topology

```
client (web3js / kit)
        │  RPC + WS
        ▼
query-filtering-service  (router; default listen 0.0.0.0:2999 / ws 3000)
   ┌────┴─────────────────────────┐
   ▼                              ▼
mb-test-validator (BASE L1)   ephemeral-validator (ER)
 RPC 7799 / WS 7800 (cfg)      RPC 8899 / WS 8900 (router default --ephemeral-url)
 - dlp, magic, vrf, acl, spl   - clones accounts from base, writes delegated accounts
```

The router exposes the `getBlockhashForAccounts` / `getClosestValidator` extensions that
both TS SDKs detect (`isRouter`/`isMagicRouter`). Undelegated accounts route to base,
delegated accounts route to ER. **Exact port wiring is the main empirical risk** and is
pinned down by the flagship spike before replication (see Build order).

A reusable shell script `examples/scripts/start-validators.sh` boots the three services,
waits for health, and writes their PIDs; `stop-validators.sh` tears them down. CI and
local runs share this script.

## Repo layout

```
examples/
  README.md                      # overview + how to run one example
  scripts/
    start-validators.sh          # boot base + ER + router, wait healthy
    stop-validators.sh
    lib.sh                       # shared helpers (wait_for_rpc, airdrop, etc.)
  <feature>-anchor/
    Anchor.toml
    Cargo.toml
    programs/<name>/src/lib.rs    # the program
    package.json                  # test deps; vitest
    tsconfig.json
    vitest.config.ts
    tests/
      web3js.test.ts
      kit.test.ts
  <feature>-pinocchio/
    Cargo.toml
    src/lib.rs                    # entrypoint program (no_std pinocchio)
    package.json
    tsconfig.json
    vitest.config.ts
    tests/
      web3js.test.ts
      kit.test.ts
```

Each example is **self-contained** (own Cargo manifest + own npm package), consuming the
local SDK crates via `path = "../../rust/..."` and the local TS SDKs via
`file:../../ts/web3js` / `file:../../ts/kit`. This keeps examples tracking unreleased SDK
changes and makes "one runner per example" a clean CI matrix entry.

## Feature → example matrix

The flagship `counter` example demonstrates the **full lifecycle** (delegate → run on ER →
commit → commit-and-undelegate → undelegate) and is the proven template. Subsequent
examples are minimal single-feature programs.

| # | Example dir | Feature(s) demonstrated | Anchor | Pinocchio |
|---|---|---|:--:|:--:|
| 1 | `counter-{anchor,pinocchio}` | `#[ephemeral]`, delegate, commit, commit+undelegate, undelegate, `#[commit]` | ✅ | ✅ |
| 2 | `actions-{anchor,pinocchio}` | `delegate_account_with_actions` / `PostDelegationActions`, `#[action]` | ✅ | ✅ |
| 3 | `ephemeral-accounts-{anchor,pinocchio}` | `#[ephemeral_accounts]` / ephemeral balance top-up & close | ✅ | ✅ |
| 4 | `access-control-{anchor,pinocchio}` | ACL permissions: create/update/delegate/commit/undelegate/close permission | ✅ | ✅ |
| 5 | `spl-{anchor,pinocchio}` | ephemeral ATA init/delegate, deposit & withdraw SPL tokens | ✅ | ✅ |
| 6 | `vrf-{anchor,pinocchio}` | `#[vrf]` / `#[vrf_callback]`, request randomness + callback (vrf-oracle) | ✅ | ✅ |
| 7 | `intent-bundle-pinocchio` | `intent_bundle` commit / commit-and-undelegate bundling | — | ✅ |

Anchor coverage of intent bundles is provided through the `ephem` intent builders inside
the `counter-anchor` advanced test rather than a separate program (YAGNI). The crank
helper is exercised implicitly by the SPL example's transfer-queue crank.

Total: 13 example directories → 13 CI runners.

## Remaining-feature implementation notes (turnkey roadmap)

Each follows the counter pattern: program (anchor + pinocchio) + `web3js.test.ts` +
`kit.test.ts` + a matrix entry in `examples.yml`. Build with `cargo build-sbf`; restart
the stack to test a rebuilt program (preloaded `--bpf-program` binaries are
non-upgradeable). Reuse the counter's `_shared.ts`, `send` (skip-preflight), auth-token
flow, and per-payer PDA seeding.

- **delegate-with-actions** — Anchor: `cpi::delegate_account_with_actions` +
  `dlp_api::args::{DelegateWithActionsArgs, PostDelegationActions}`; account struct uses
  `#[action]` (injects `escrow_auth` / `escrow`). Pinocchio: enable the
  `delegation-actions` feature on `ephemeral-rollups-pinocchio`, use
  `instruction::delegate_with_actions`. Demonstrate a post-delegation action (e.g. a CPI
  scheduled to run on commit).
- **ephemeral-accounts** — Anchor `#[ephemeral_accounts]` with `eph` / `sponsor`
  field markers (gas-sponsored ephemeral balances). TS: `topUpEphemeralBalance` /
  `closeEphemeralBalance` + `escrowPdaFromEscrowAuthority`.
- **access-control** — uses the preloaded permission program
  (`ACLseoPoyC3cBqoUtkbjZ4aDrkurZW86v19pXz2XQnp1`), `sdk::access_control` (feature
  `access-control`). TS: the `permission-program` instructions
  (`createPermission`/`update`/`delegate`/`commit`/`undelegate`/`close`) +
  `permissionPdaFromAccount`.
- **spl** — ephemeral ATAs via `sdk::spl` builders (feature `spl`) against the
  preloaded ephemeral SPL token program (`SPLxh1…`). TS: the
  `ephemeral-spl-token-program` instructions (init/delegate ATA, deposit, withdraw).
- **vrf** — Anchor `#[vrf]` / `#[vrf_callback]` (`ephemeral-vrf-sdk`); request randomness
  and handle the callback. Requires running the `vrf-oracle` binary alongside the stack
  (add it to `start-validators.sh` for this example).
- **intent-bundle** — Pinocchio `intent_bundle` (commit / commit-and-undelegate
  bundling); Anchor analogue via `ephem` intent builders.

## Per-example program shape (flagship: counter)

**Anchor** (`counter-anchor`): a module annotated `#[ephemeral]` with instructions
`initialize`, `increment`, `delegate` (`#[delegate]` accounts), `commit` and
`commit_and_undelegate` (`#[commit]` accounts), `undelegate` (delegation-program callback).
PDA seed `b"counter"`. State: `{ count: u64 }`.

**Pinocchio** (`counter-pinocchio`): single `process_instruction` dispatching on a 1-byte
tag to the same logical operations, using `instruction::{delegate_account, commit_accounts,
commit_and_undelegate_accounts, undelegate}` helpers.

Both share the same on-chain account layout so the TS tests differ only in program ID and
instruction encoding helpers.

## TypeScript test design

Each test follows the identical arc, asserting observable state transitions:

1. Connect to the **router** endpoint (both SDKs auto-detect router mode).
2. Airdrop to payer on base; `initialize` the PDA on base; assert `count == 0`.
3. `increment` on base; assert `count == 1`.
4. `delegate` the PDA (program CPI to dlp); assert owner becomes the delegation program.
5. `increment` routed to **ER**; assert ER state advances while base is frozen.
6. `commit`; assert base state catches up to ER.
7. `commit_and_undelegate` (or `undelegate`); assert owner returns to the program and
   final `count` is consistent across base.

- **web3.js test**: `@solana/web3.js` + `@magicblock-labs/ephemeral-rollups-sdk`
  (`ConnectionMagicRouter`, `delegateBufferPda`, magic-program commit instructions).
- **kit test**: `@solana/kit` + `@magicblock-labs/ephemeral-rollups-kit` (`Connection`,
  instruction builders). Anchor examples use the generated IDL + a thin client; pinocchio
  examples hand-encode instruction data.

A shared `examples/scripts/lib.sh` and a small TS `tests/_shared/` helper (per example,
copied not imported, to keep examples standalone) provide airdrop/confirm utilities.

## CI design

New workflow `.github/workflows/examples.yml` (triggered on PR/push touching `examples/**`
and the SDK sources). Strategy matrix with one entry per example directory:

```yaml
strategy:
  fail-fast: false
  matrix:
    example:
      - counter-anchor
      - counter-pinocchio
      - actions-anchor
      ...   # 13 entries
```

Each job:
1. checkout; install rust 1.93.1, solana v3.1.10, anchor 0.31.1 (anchor jobs only), node 23.
2. `npm install -g @magicblock-labs/ephemeral-validator@latest`.
3. Build the example program (`anchor build` or `cargo build-sbf`).
4. `examples/scripts/start-validators.sh` (base + ER + router), deploy program to base.
5. `cd examples/<example> && npm install && npm test` (runs both web3js + kit vitest files).
6. Always `stop-validators.sh` + upload validator logs on failure.

A reusable composite action `examples/.ci/setup` factors out steps 1–2 to keep the matrix
DRY. Rust fmt/clippy for example programs run in a separate lightweight lint job.

## Build / validation order (de-risk first)

1. **Spike**: `counter-anchor` end-to-end **locally** (validators are installed on this
   machine) — pins exact port wiring, router config, and the start/stop scripts. This is
   the riskiest unknown; prove it before anything else.
2. `counter-anchor` web3js + kit tests green locally → write `examples.yml` for this one
   example → confirm CI green.
3. `counter-pinocchio` (reuse scripts + CI template).
4. Replicate per feature (matrix grows one pair at a time): actions, ephemeral-accounts,
   access-control, spl, vrf, intent-bundle.
5. `examples/README.md` + root README link.

Each step ends with verified-green tests before moving on; no example is declared done
until both its TS tests pass against the live stack.

## Risks & open questions

- **Router port wiring** — the exact base/ER/router port assignment and whether
  `query-filtering-service` needs an explicit base URL is resolved empirically in the spike.
- **ER warm-up / commit latency** — tests must poll with timeouts, not fixed sleeps;
  helpers encapsulate this.
- **anchor build time in CI** — mitigated by `Swatinem/rust-cache` and per-example jobs.
- **Pinocchio + IDL** — pinocchio programs have no IDL; kit/web3js tests hand-encode.
```
