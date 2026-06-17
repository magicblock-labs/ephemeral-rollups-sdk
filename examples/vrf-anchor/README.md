# vrf-anchor

A minimal **Anchor** example of verifiable randomness (VRF):

```
initialize → request (asks the oracle queue) → consume (oracle callback writes a random 1..=100)
```

- `#[vrf]` injects `program_identity` / `vrf_program` / `slot_hashes` and the
  `invoke_signed_vrf` helper used to issue a scoped randomness request.
- `#[vrf_callback]` injects the `vrf_program_identity` signer that proves the callback
  came from the VRF program for this program.

The randomness request is fulfilled by the `vrf-oracle` (from the validator package),
which must run subscribed to the layer where the request lands — here the base layer.
`start-validators.sh` starts it when `START_VRF_ORACLE=1`.

| Path | What |
|------|------|
| `src/lib.rs` | the program (`initialize` / `request` / `consume` callback) |
| `tests/web3js.test.ts` | `@solana/web3.js` + `@magicblock-labs/ephemeral-rollups-sdk` |
| `tests/kit.test.ts` | `@solana/kit` + `@magicblock-labs/ephemeral-rollups-kit` |

## Running

```bash
cd examples/vrf-anchor && cargo build-sbf && cd -
START_VRF_ORACLE=1 examples/scripts/start-validators.sh examples/vrf-anchor/target/deploy
cd examples/vrf-anchor && yarn install && yarn test && cd -
examples/scripts/stop-validators.sh
```

The VRF program (`Vrf1RNUjXmQGjmQrQLvJHs9SNkvDJEsRVFPkfSQUwGz`) and oracle queue are
preloaded by `mb-test-validator`.
