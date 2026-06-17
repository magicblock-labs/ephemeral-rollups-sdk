# spl

Demonstrates the SDK's **ephemeral SPL token** feature against the preloaded ephemeral
SPL token program (`SPLxh1LVZzEkX99H6rqYizhytLWPZVV296zyYDPagv2`): set up an SPL mint,
initialize the global vault (+ vault ATA + rent PDA), create an **ephemeral ATA**, and
deposit SPL tokens into it so they can be operated on the rollup.

Client-driven (no custom program). The mint is created with the standard SPL token
client; the ephemeral-ATA instructions come from the ER SDKs.

| Path | What |
|------|------|
| `tests/web3js.test.ts` | `@solana/web3.js` + `@magicblock-labs/ephemeral-rollups-sdk` + `@solana/spl-token` |
| `tests/kit.test.ts` | `@solana/kit` + `@magicblock-labs/ephemeral-rollups-kit` (mint setup via `@solana/spl-token`, as there is no kit-v4 SPL-token client) |

## Running

```bash
examples/scripts/start-validators.sh        # only the base layer is needed here
cd examples/spl && npm install && npm test && cd -
examples/scripts/stop-validators.sh
```
