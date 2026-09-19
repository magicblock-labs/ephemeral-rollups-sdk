# @magicblock-labs/ephemeral-rollups-sdk

Transaction helpers, delegation instructions, PDAs, token operations, and
access control for `@solana/web3.js` 1.x.

```bash
npm install @magicblock-labs/ephemeral-rollups-sdk
```

```ts
import { ConnectionMagicRouter } from "@magicblock-labs/ephemeral-rollups-sdk";

// routerUrl must support Magic Router RPC extensions.
const connection = new ConnectionMagicRouter(routerUrl, "confirmed");
```

- Set the fee payer and instructions before requesting a routed blockhash:
  routing uses the transaction's writable accounts, including the fee payer.
- `prepareTransaction` changes the recent blockhash; call it before signing.
  Legacy `sendTransaction` also fetches a routed blockhash before signing with
  the supplied signers.
- `VersionedTransaction` uses web3.js's standard send path; it does not receive
  the legacy path's automatic routed-blockhash preparation.

Use the [Kit package](../kit/README.md) for `@solana/kit` types and connections.

[Integration guides](https://docs.magicblock.gg/)
