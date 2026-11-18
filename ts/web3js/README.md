# @magicblock-labs/ephemeral-rollups-sdk

Transaction helpers, delegation instructions, PDAs, token operations, and
access control for `@solana/web3.js` 1.x.

```bash
npm install @magicblock-labs/ephemeral-rollups-sdk
```

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
