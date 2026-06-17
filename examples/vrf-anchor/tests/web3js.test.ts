// VRF example using @solana/web3.js + @magicblock-labs/ephemeral-rollups-sdk.
// Requests verifiable randomness; the vrf-oracle fulfils it via the `consume` callback.
import { describe, it, expect, beforeAll } from "vitest";
import {
  Connection,
  Keypair,
  PublicKey,
  SystemProgram,
  Transaction,
  TransactionInstruction,
} from "@solana/web3.js";
import {
  BASE_RPC_URL,
  RANDOM_SEED,
  PROGRAM_ID_BYTES,
  VRF_PROGRAM_ID_STR,
  DEFAULT_TEST_QUEUE_STR,
  SLOT_HASHES_STR,
  VRF_IDENTITY_SEED,
  ixDiscriminator,
  decodeValue,
  waitFor,
} from "./_shared";

const PROGRAM_ID = new PublicKey(PROGRAM_ID_BYTES);
const VRF_PROGRAM_ID = new PublicKey(VRF_PROGRAM_ID_STR);
const ORACLE_QUEUE = new PublicKey(DEFAULT_TEST_QUEUE_STR);
const SLOT_HASHES = new PublicKey(SLOT_HASHES_STR);
const [PROGRAM_IDENTITY] = PublicKey.findProgramAddressSync([VRF_IDENTITY_SEED], PROGRAM_ID);

const base = new Connection(BASE_RPC_URL, "confirmed");
const payer = Keypair.generate();
const [RANDOM_PDA] = PublicKey.findProgramAddressSync(
  [RANDOM_SEED, payer.publicKey.toBytes()],
  PROGRAM_ID,
);

async function send(ixs: TransactionInstruction[]) {
  const tx = new Transaction().add(...ixs);
  const { blockhash, lastValidBlockHeight } = await base.getLatestBlockhash();
  tx.recentBlockhash = blockhash;
  tx.lastValidBlockHeight = lastValidBlockHeight;
  tx.feePayer = payer.publicKey;
  tx.sign(payer);
  const sig = await base.sendRawTransaction(tx.serialize());
  await base.confirmTransaction({ signature: sig, blockhash, lastValidBlockHeight }, "confirmed");
  return sig;
}

describe("vrf-anchor (web3.js)", () => {
  beforeAll(async () => {
    const sig = await base.requestAirdrop(payer.publicKey, 5_000_000_000);
    const bh = await base.getLatestBlockhash();
    await base.confirmTransaction({ signature: sig, ...bh }, "confirmed");
  });

  it("requests randomness and consumes it via the oracle callback", async () => {
    // initialize
    await send([
      new TransactionInstruction({
        programId: PROGRAM_ID,
        keys: [
          { pubkey: payer.publicKey, isSigner: true, isWritable: true },
          { pubkey: RANDOM_PDA, isSigner: false, isWritable: true },
          { pubkey: SystemProgram.programId, isSigner: false, isWritable: false },
        ],
        data: ixDiscriminator("initialize"),
      }),
    ]);
    expect(decodeValue((await base.getAccountInfo(RANDOM_PDA))!.data)).toBe(0);

    // request randomness (client_seed = 7)
    await send([
      new TransactionInstruction({
        programId: PROGRAM_ID,
        keys: [
          { pubkey: payer.publicKey, isSigner: true, isWritable: true },
          { pubkey: RANDOM_PDA, isSigner: false, isWritable: true },
          { pubkey: ORACLE_QUEUE, isSigner: false, isWritable: true },
          { pubkey: PROGRAM_IDENTITY, isSigner: false, isWritable: false },
          { pubkey: VRF_PROGRAM_ID, isSigner: false, isWritable: false },
          { pubkey: SLOT_HASHES, isSigner: false, isWritable: false },
          { pubkey: SystemProgram.programId, isSigner: false, isWritable: false },
        ],
        data: Buffer.concat([ixDiscriminator("request"), Buffer.from([7])]),
      }),
    ]);

    // the oracle fulfils the request by calling `consume`, setting value in 1..=100
    const value = await waitFor(
      async () => {
        const v = decodeValue((await base.getAccountInfo(RANDOM_PDA))!.data);
        return v !== 0 ? v : false;
      },
      { timeoutMs: 60_000, intervalMs: 1_000 },
    );
    expect(value).toBeGreaterThanOrEqual(1);
    expect(value).toBeLessThanOrEqual(100);
  });
});
