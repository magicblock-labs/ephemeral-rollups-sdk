# Magic Resolver

`magic-resolver` selects a Solana or Ephemeral Rollup RPC client from account
delegation status.

- `Resolver::new` takes base-chain HTTP/WebSocket configuration, cache size, and commitment.
- `resolve` routes a single account; `resolve_for_transaction` inspects writable accounts.
- Transactions with writable accounts delegated to different validators are rejected.
  Undelegated writable accounts do not themselves cause a conflict; if none are
  delegated, the resolver returns the base-chain client.
- Cached delegation status is used only while its WebSocket subscription is active;
  otherwise status is fetched from the base chain. A missing validator route is an error.

Resolution chooses a client; it does not guarantee transaction execution will
succeed or prevent delegation state from changing before submission.

[API reference](https://docs.rs/magic-resolver) ·
[Integration guides](https://docs.magicblock.gg/)
