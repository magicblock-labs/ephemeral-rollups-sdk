// Framework-agnostic helpers shared by the web3.js and kit tests.
import { readFileSync } from "node:fs";
import { fileURLToPath } from "node:url";
import { dirname, join } from "node:path";
import { sha256 } from "@noble/hashes/sha256";

const here = dirname(fileURLToPath(import.meta.url));

/** Base layer (mb-test-validator) RPC. */
export const BASE_RPC_URL = process.env.BASE_RPC_URL ?? "http://127.0.0.1:8899";
/** Base layer WebSocket (solana-test-validator serves WS on rpc-port + 1). */
export const BASE_WS_URL = process.env.BASE_WS_URL ?? "ws://127.0.0.1:8900";
/** Rollup endpoint: the query-filtering-service in front of the ER. */
export const ROUTER_RPC_URL =
  process.env.ROUTER_RPC_URL ?? "http://127.0.0.1:2999";
export const ROUTER_WS_URL = process.env.ROUTER_WS_URL ?? "ws://127.0.0.1:3000";

/**
 * Identity of the ephemeral validator that accounts are delegated to. The local
 * `ephemeral-validator` runs with this well-known dev identity; on a real network
 * you would resolve the target validator dynamically.
 */
export const ER_VALIDATOR_IDENTITY =
  process.env.ER_VALIDATOR_IDENTITY ??
  "mAGicPQYBMvcYveUZA5F5UNNwyHvfYh5xkLS2Fr1mev";

/** Seed of the counter PDA (matches `COUNTER_SEED` in the program). */
export const COUNTER_SEED = Buffer.from("counter");

// Well-known program ids (string form, for use with @solana/kit `address()`).
export const DELEGATION_PROGRAM_ID_STR =
  "DELeGGvXpWV2fqJUhqcF5ZSYMS4JTLjteaAMARRSaeSh";
export const MAGIC_PROGRAM_ID_STR =
  "Magic11111111111111111111111111111111111111";
export const MAGIC_CONTEXT_ID_STR =
  "MagicContext1111111111111111111111111111111";
export const SYSTEM_PROGRAM_ID_STR = "11111111111111111111111111111111";

/** Program id, read from the committed program keypair so it always matches the build. */
export const PROGRAM_ID_BYTES: Uint8Array = (() => {
  const kp = JSON.parse(
    readFileSync(join(here, "..", "program-keypair.json"), "utf8"),
  ) as number[];
  // Solana keypair files store [secret(32) || public(32)]; take the public half.
  return Uint8Array.from(kp.slice(32));
})();

/** Anchor instruction discriminator: first 8 bytes of sha256("global:<ix>"). */
export function ixDiscriminator(name: string): Buffer {
  return Buffer.from(sha256(`global:${name}`)).subarray(0, 8);
}

/** Anchor account discriminator: first 8 bytes of sha256("account:<Name>"). */
export function accountDiscriminator(name: string): Buffer {
  return Buffer.from(sha256(`account:${name}`)).subarray(0, 8);
}

/** Decode the `count` field (u64 LE) from a Counter account's raw data. */
export function decodeCount(data: Uint8Array): bigint {
  // 8-byte anchor discriminator, then u64 little-endian.
  const view = new DataView(data.buffer, data.byteOffset + 8, 8);
  return view.getBigUint64(0, true);
}

/** Poll `fn` until it returns a truthy value or the timeout elapses. */
export async function waitFor<T>(
  fn: () => Promise<T | undefined | null | false>,
  { timeoutMs = 60_000, intervalMs = 1_000 } = {},
): Promise<T> {
  const deadline = Date.now() + timeoutMs;
  for (;;) {
    const result = await fn();
    if (result) return result;
    if (Date.now() > deadline) throw new Error("waitFor: timed out");
    await new Promise((r) => setTimeout(r, intervalMs));
  }
}
