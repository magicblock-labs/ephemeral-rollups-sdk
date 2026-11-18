# @magicblock-labs/ephemeral-rollups-kit

Transaction helpers, delegation instructions, PDAs, token operations, and
access control for `@solana/kit` 4.x.

```bash
npm install @magicblock-labs/ephemeral-rollups-kit
```

```ts
import { Connection } from "@magicblock-labs/ephemeral-rollups-kit";

// rpcUrl is your cluster or Magic Router HTTP endpoint.
const connection = await Connection.create(rpcUrl);
```

- Use async `Connection.create`, not `new Connection`. It detects router support;
  pass a second URL if WebSocket access uses a different endpoint.
- `getLatestBlockhashForTransaction` uses writable accounts to request a routed
  blockhash when supported; otherwise it uses the cluster's standard RPC.
- `sendTransaction` defaults to `skipPreflight: true`; set it explicitly if you
  need preflight. Submission and confirmation are separate operations.

Use the [web3.js package](../web3js/README.md) for `PublicKey` and web3.js
transaction types; the two packages are not drop-in replacements.

## SPL token utilities

The package exports `delegateSpl`, `withdrawSpl`, and `transferSpl`, plus the
lower-level PDA and instruction builders. Inputs use Kit `Address` values and
outputs use Kit `Instruction` values. Await PDA derivations and async builders.

```ts
import {
  delegateSpl,
  withdrawSpl,
} from "@magicblock-labs/ephemeral-rollups-kit";

// owner and mint are Kit Address values; amounts are in raw token units.
const depositInstructions = await delegateSpl(owner, mint, 1_000n, {
  initVaultIfMissing: true,
  initAtasIfMissing: true,
});
const withdrawalInstructions = await withdrawSpl(owner, mint, 1_000n, {
  initAtasIfMissing: true,
});
```

Both helpers use the idempotent shuttle flow by default. Set `idempotent: false`
for the legacy direct flow. Pass `tokenProgram` to use another token program;
it is used consistently for associated token addresses, setup, and transfers.
The default remains `TOKEN_PROGRAM_ID`.

For explicit control, use `deriveEphemeralAta`, `deriveVault`, `deriveVaultAta`,
`initEphemeralAtaIx`, `initVaultIx`, `initVaultAtaIx`, `depositSplTokensIx`,
`delegateEphemeralAtaIx`, `withdrawSplIx`, and `undelegateIx`.
`depositSplTokensIx` and `delegateEphemeralAtaIx` are aliases of the existing
`transferToVaultIx` and `delegateIx` helpers. Account encoding and decoding are
available through `getEphemeralAtaCodec` and `getGlobalVaultCodec`.

[Integration guides](https://docs.magicblock.gg/)

## SDK versioning for Validator v1 and v2

The Rust and TypeScript SDKs share a version number.

| SDK version line | Validator support | Guidance |
| --- | --- | --- |
| `0.17.x` | Validator v1 | — |
| `0.18.x` | Validator v1 and v2 | Users are encouraged to migrate to `0.20.x` or later. |
| `0.19.x` | Reserved for possible v1-compatible releases with breaking SDK changes. | — |
| `0.20.x` and later | Validator v2 release line | Recommended for new integrations. |

`0.20.0-alpha.1` was the first release in the Validator v2 line. Version `1.0.0`
is reserved for the planned major SDK redesign.
