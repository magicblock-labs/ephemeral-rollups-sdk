import { readFileSync } from "node:fs";
import { fileURLToPath } from "node:url";
import { dirname, join } from "node:path";
import { sha256 } from "@noble/hashes/sha256";

const here = dirname(fileURLToPath(import.meta.url));

export const BASE_RPC_URL = process.env.BASE_RPC_URL ?? "http://127.0.0.1:8899";
export const BASE_WS_URL = process.env.BASE_WS_URL ?? "ws://127.0.0.1:8900";
export const ROUTER_RPC_URL =
  process.env.ROUTER_RPC_URL ?? "http://127.0.0.1:2999";
export const ROUTER_WS_URL = process.env.ROUTER_WS_URL ?? "ws://127.0.0.1:3000";

export const ER_VALIDATOR_IDENTITY =
  process.env.ER_VALIDATOR_IDENTITY ??
  "mAGicPQYBMvcYveUZA5F5UNNwyHvfYh5xkLS2Fr1mev";

export const TREASURY_SEED = Buffer.from("treasury");
export const GAME_SEED = Buffer.from("game");
export const EPHEMERAL_VAULT_ID_STR =
  "MagicVau1t999999999999999999999999999999999";
export const MAGIC_PROGRAM_ID_STR =
  "Magic11111111111111111111111111111111111111";
export const SYSTEM_PROGRAM_ID_STR = "11111111111111111111111111111111";

export const PROGRAM_ID_BYTES: Uint8Array = (() => {
  const kp = JSON.parse(
    readFileSync(join(here, "..", "program-keypair.json"), "utf8"),
  ) as number[];
  return Uint8Array.from(kp.slice(32));
})();

export function ixDiscriminator(name: string): Buffer {
  return Buffer.from(sha256(`global:${name}`)).subarray(0, 8);
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
