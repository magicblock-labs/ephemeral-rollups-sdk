// Shared helpers for the VRF example tests.
import { readFileSync } from "node:fs";
import { fileURLToPath } from "node:url";
import { dirname, join } from "node:path";
import { sha256 } from "@noble/hashes/sha256";

const here = dirname(fileURLToPath(import.meta.url));

export const BASE_RPC_URL = process.env.BASE_RPC_URL ?? "http://127.0.0.1:8899";
export const BASE_WS_URL = process.env.BASE_WS_URL ?? "ws://127.0.0.1:8900";

export const RANDOM_SEED = Buffer.from("random");

// Well-known VRF addresses (stable; preloaded by mb-test-validator).
export const VRF_PROGRAM_ID_STR = "Vrf1RNUjXmQGjmQrQLvJHs9SNkvDJEsRVFPkfSQUwGz";
export const DEFAULT_TEST_QUEUE_STR =
  "GKE6d7iv8kCBrsxr78W3xVdjGLLLJnxsGiuzrsZCGEvb";
export const SLOT_HASHES_STR = "SysvarS1otHashes111111111111111111111111111";
export const SYSTEM_PROGRAM_ID_STR = "11111111111111111111111111111111";
/** Seed of the per-program VRF identity PDA (`["identity"]` under the program). */
export const VRF_IDENTITY_SEED = Buffer.from("identity");

export const PROGRAM_ID_BYTES: Uint8Array = (() => {
  const kp = JSON.parse(
    readFileSync(join(here, "..", "program-keypair.json"), "utf8"),
  ) as number[];
  return Uint8Array.from(kp.slice(32));
})();

export function ixDiscriminator(name: string): Buffer {
  return Buffer.from(sha256(`global:${name}`)).subarray(0, 8);
}

/** Decode the `value` (u8) of the Random account (8-byte anchor discriminator first). */
export function decodeValue(data: Uint8Array): number {
  return data[8];
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
