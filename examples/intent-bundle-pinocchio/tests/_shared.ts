// Framework-agnostic helpers shared by the web3.js and kit tests.
//
// Unlike the Anchor example, this Pinocchio program uses single-byte instruction
// tags (no 8-byte discriminator) and stores the counter as a bare u64 at offset 0
// (no account discriminator).
import { readFileSync } from "node:fs";
import { fileURLToPath } from "node:url";
import { dirname, join } from "node:path";

const here = dirname(fileURLToPath(import.meta.url));

export const BASE_RPC_URL = process.env.BASE_RPC_URL ?? "http://127.0.0.1:8899";
export const BASE_WS_URL = process.env.BASE_WS_URL ?? "ws://127.0.0.1:8900";
export const ROUTER_RPC_URL =
  process.env.ROUTER_RPC_URL ?? "http://127.0.0.1:2999";
export const ROUTER_WS_URL = process.env.ROUTER_WS_URL ?? "ws://127.0.0.1:3000";

export const ER_VALIDATOR_IDENTITY =
  process.env.ER_VALIDATOR_IDENTITY ??
  "mAGicPQYBMvcYveUZA5F5UNNwyHvfYh5xkLS2Fr1mev";

export const COUNTER_SEED = Buffer.from("counter");

export const DELEGATION_PROGRAM_ID_STR =
  "DELeGGvXpWV2fqJUhqcF5ZSYMS4JTLjteaAMARRSaeSh";
export const MAGIC_PROGRAM_ID_STR =
  "Magic11111111111111111111111111111111111111";
export const MAGIC_CONTEXT_ID_STR =
  "MagicContext1111111111111111111111111111111";
export const SYSTEM_PROGRAM_ID_STR = "11111111111111111111111111111111";

// Instruction tags (must match the program's dispatch in src/lib.rs).
export const TAG = {
  initialize: 0,
  increment: 1,
  delegate: 2,
  commit: 3,
  commitAndUndelegate: 4,
} as const;

export const PROGRAM_ID_BYTES: Uint8Array = (() => {
  const kp = JSON.parse(
    readFileSync(join(here, "..", "program-keypair.json"), "utf8"),
  ) as number[];
  return Uint8Array.from(kp.slice(32));
})();

/** Decode the counter (u64 LE) stored at offset 0. */
export function decodeCount(data: Uint8Array): bigint {
  const view = new DataView(data.buffer, data.byteOffset, 8);
  return view.getBigUint64(0, true);
}

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
