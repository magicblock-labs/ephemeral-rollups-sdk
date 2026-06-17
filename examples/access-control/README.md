# access-control

Demonstrates the SDK's **access-control** feature: managing permission accounts on
the permission program (`ACLseoPoyC3cBqoUtkbjZ4aDrkurZW86v19pXz2XQnp1`, preloaded by
`mb-test-validator`).

Unlike the counter examples this is **client-driven** (no custom program): the
access-control rust API is a set of instruction builders targeting the permission
program, and both TypeScript SDKs expose the same builders. The tests create a
permission (with the payer as the sole authority member), update its member set, and
close it — asserting the permission PDA appears, persists, and is removed.

| Path | What |
|------|------|
| `tests/web3js.test.ts` | `@solana/web3.js` + `@magicblock-labs/ephemeral-rollups-sdk` |
| `tests/kit.test.ts` | `@solana/kit` + `@magicblock-labs/ephemeral-rollups-kit` |

## Running

```bash
examples/scripts/start-validators.sh        # only the base layer is needed here
cd examples/access-control && yarn install && yarn test && cd -
examples/scripts/stop-validators.sh
```
