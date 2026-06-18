// VRF example using @solana/kit + @magicblock-labs/ephemeral-rollups-kit.
import { describe, it, expect, beforeAll } from "vitest";
import {
  address,
  appendTransactionMessageInstruction,
  createTransactionMessage,
  generateKeyPairSigner,
  getAddressEncoder,
  getAddressDecoder,
  getProgramDerivedAddress,
  getBase64Encoder,
  lamports,
  pipe,
  setTransactionMessageFeePayerSigner,
  setTransactionMessageLifetimeUsingBlockhash,
  AccountRole,
  type Address,
  type IInstruction,
  type KeyPairSigner,
} from "@solana/kit";
import { Connection } from "@magicblock-labs/ephemeral-rollups-kit";
import {
  BASE_RPC_URL,
  BASE_WS_URL,
  RANDOM_SEED,
  PROGRAM_ID_BYTES,
  VRF_PROGRAM_ID_STR,
  DEFAULT_TEST_QUEUE_STR,
  SLOT_HASHES_STR,
  SYSTEM_PROGRAM_ID_STR,
  VRF_IDENTITY_SEED,
  ixDiscriminator,
  decodeValue,
  waitFor,
} from "./_shared";

const addressEncoder = getAddressEncoder();
const base64 = getBase64Encoder();
const PROGRAM_ID = getAddressDecoder().decode(PROGRAM_ID_BYTES);
const VRF_PROGRAM_ID = address(VRF_PROGRAM_ID_STR);
const ORACLE_QUEUE = address(DEFAULT_TEST_QUEUE_STR);
const SLOT_HASHES = address(SLOT_HASHES_STR);
const SYSTEM_PROGRAM_ID = address(SYSTEM_PROGRAM_ID_STR);

let base: Connection;
let payer: KeyPairSigner;
let RANDOM_PDA: Address;
let PROGRAM_IDENTITY: Address;

function meta(addr: Address, role: AccountRole) {
  return { address: addr, role };
}

async function send(instruction: IInstruction) {
  const { value: blockhash } = await base.rpc.getLatestBlockhash().send();
  const tx = pipe(
    createTransactionMessage({ version: 0 }),
    (m) => setTransactionMessageFeePayerSigner(payer, m),
    (m) => setTransactionMessageLifetimeUsingBlockhash(blockhash, m),
    (m) => appendTransactionMessageInstruction(instruction, m),
  );
  const sig = await base.sendTransaction(tx, [payer.keyPair]);
  // Poll signature status (more robust than the WS-based confirmTransaction when the
  // oracle is also driving the base layer).
  await waitFor(async () => {
    const { value } = await base.rpc.getSignatureStatuses([sig]).send();
    const s = value[0];
    return (
      s?.confirmationStatus === "confirmed" ||
      s?.confirmationStatus === "finalized"
    );
  });
  return sig;
}

async function getValue(): Promise<number | null> {
  const { value } = await base.rpc
    .getAccountInfo(RANDOM_PDA, { encoding: "base64" })
    .send();
  if (!value) return null;
  return decodeValue(new Uint8Array(base64.encode(value.data[0])));
}

describe("vrf-anchor (kit)", () => {
  beforeAll(async () => {
    payer = await generateKeyPairSigner();
    [RANDOM_PDA] = await getProgramDerivedAddress({
      programAddress: PROGRAM_ID,
      seeds: [RANDOM_SEED, addressEncoder.encode(payer.address)],
    });
    [PROGRAM_IDENTITY] = await getProgramDerivedAddress({
      programAddress: PROGRAM_ID,
      seeds: [VRF_IDENTITY_SEED],
    });
    base = await Connection.create(BASE_RPC_URL, BASE_WS_URL);
    const airdrop = await base.rpc
      .requestAirdrop(payer.address, lamports(5_000_000_000n))
      .send();
    await waitFor(async () => {
      const { value } = await base.rpc.getSignatureStatuses([airdrop]).send();
      const s = value[0];
      return (
        s?.confirmationStatus === "confirmed" ||
        s?.confirmationStatus === "finalized"
      );
    });
  });

  it("requests randomness and consumes it via the oracle callback", async () => {
    await send({
      programAddress: PROGRAM_ID,
      accounts: [
        meta(payer.address, AccountRole.WRITABLE_SIGNER),
        meta(RANDOM_PDA, AccountRole.WRITABLE),
        meta(SYSTEM_PROGRAM_ID, AccountRole.READONLY),
      ],
      data: new Uint8Array(ixDiscriminator("initialize")),
    });
    expect(await getValue()).toBe(0);

    const requestData = new Uint8Array(9);
    requestData.set(ixDiscriminator("request"), 0);
    requestData[8] = 7; // client_seed
    await send({
      programAddress: PROGRAM_ID,
      accounts: [
        meta(payer.address, AccountRole.WRITABLE_SIGNER),
        meta(RANDOM_PDA, AccountRole.WRITABLE),
        meta(ORACLE_QUEUE, AccountRole.WRITABLE),
        meta(PROGRAM_IDENTITY, AccountRole.READONLY),
        meta(VRF_PROGRAM_ID, AccountRole.READONLY),
        meta(SLOT_HASHES, AccountRole.READONLY),
        meta(SYSTEM_PROGRAM_ID, AccountRole.READONLY),
      ],
      data: requestData,
    });

    const value = await waitFor(
      async () => {
        const v = await getValue();
        return v && v !== 0 ? v : false;
      },
      { timeoutMs: 60_000, intervalMs: 1_000 },
    );
    expect(value).toBeGreaterThanOrEqual(1);
    expect(value).toBeLessThanOrEqual(100);
  });
});
