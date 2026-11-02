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

[Integration guides](https://docs.magicblock.gg/)
