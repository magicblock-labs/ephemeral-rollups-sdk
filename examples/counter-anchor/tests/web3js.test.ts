// Counter lifecycle test using @solana/web3.js + @magicblock-labs/ephemeral-rollups-sdk.
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
  DELEGATION_PROGRAM_ID,
  MAGIC_PROGRAM_ID,
  MAGIC_CONTEXT_ID,
  GetCommitmentSignature,
  getAuthToken,
  delegationRecordPdaFromDelegatedAccount,
  delegationMetadataPdaFromDelegatedAccount,
  delegateBufferPdaFromDelegatedAccountAndOwnerProgram,
} from "@magicblock-labs/ephemeral-rollups-sdk";
import nacl from "tweetnacl";
import {
  BASE_RPC_URL,
  ROUTER_RPC_URL,
  ROUTER_WS_URL,
  COUNTER_SEED,
  PROGRAM_ID_BYTES,
  ER_VALIDATOR_IDENTITY,
  ixDiscriminator,
  decodeCount,
  waitFor,
} from "./_shared";

const PROGRAM_ID = new PublicKey(PROGRAM_ID_BYTES);

const base = new Connection(BASE_RPC_URL, "confirmed");
// The ephemeral connection (set up in beforeAll once we have an auth token for the
// query-filtering-service).
let ephemeral: Connection;

const payer = Keypair.generate();
// Counter PDA is seeded per payer so each test file gets an isolated account.
const [COUNTER_PDA] = PublicKey.findProgramAddressSync(
  [COUNTER_SEED, payer.publicKey.toBytes()],
  PROGRAM_ID,
);

function ix(
  name: string,
  keys: { pubkey: PublicKey; isSigner: boolean; isWritable: boolean }[],
  args: Buffer = Buffer.alloc(0),
) {
  return new TransactionInstruction({
    programId: PROGRAM_ID,
    keys,
    data: Buffer.concat([ixDiscriminator(name), args]),
  });
}

async function send(conn: Connection, instruction: TransactionInstruction) {
  const tx = new Transaction().add(instruction);
  const { blockhash, lastValidBlockHeight } = await conn.getLatestBlockhash();
  tx.recentBlockhash = blockhash;
  tx.lastValidBlockHeight = lastValidBlockHeight;
  tx.feePayer = payer.publicKey;
  tx.sign(payer);
  // ER transactions use a non-delegated fee payer, which the ER's preflight
  // verification rejects even though execution succeeds; skip preflight.
  const sig = await conn.sendRawTransaction(tx.serialize(), {
    skipPreflight: true,
  });
  await conn.confirmTransaction(
    { signature: sig, blockhash, lastValidBlockHeight },
    "confirmed",
  );
  return sig;
}

async function getCount(conn: Connection): Promise<bigint | null> {
  const acc = await conn.getAccountInfo(COUNTER_PDA);
  return acc ? decodeCount(acc.data) : null;
}

describe("counter-anchor (web3.js)", () => {
  beforeAll(async () => {
    const sig = await base.requestAirdrop(payer.publicKey, 5_000_000_000);
    const bh = await base.getLatestBlockhash();
    await base.confirmTransaction({ signature: sig, ...bh }, "confirmed");

    // The query-filtering-service requires a JWT obtained by signing a challenge.
    const { token } = await getAuthToken(
      ROUTER_RPC_URL,
      payer.publicKey,
      async (msg) => nacl.sign.detached(msg, payer.secretKey),
    );
    ephemeral = new Connection(`${ROUTER_RPC_URL}?token=${token}`, {
      wsEndpoint: `${ROUTER_WS_URL}?token=${token}`,
      commitment: "confirmed",
    });
  });

  it("runs delegate -> increment-on-ER -> commit -> undelegate", async () => {
    // 1. initialize on base
    await send(
      base,
      ix("initialize", [
        { pubkey: payer.publicKey, isSigner: true, isWritable: true },
        { pubkey: COUNTER_PDA, isSigner: false, isWritable: true },
        { pubkey: SystemProgram.programId, isSigner: false, isWritable: false },
      ]),
    );
    expect(await getCount(base)).toBe(0n);

    // 2. increment on base
    await send(
      base,
      ix("increment", [
        { pubkey: payer.publicKey, isSigner: true, isWritable: true },
        { pubkey: COUNTER_PDA, isSigner: false, isWritable: true },
      ]),
    );
    expect(await getCount(base)).toBe(1n);

    // 3. delegate the PDA
    const buffer = delegateBufferPdaFromDelegatedAccountAndOwnerProgram(
      COUNTER_PDA,
      PROGRAM_ID,
    );
    const record = delegationRecordPdaFromDelegatedAccount(COUNTER_PDA);
    const metadata = delegationMetadataPdaFromDelegatedAccount(COUNTER_PDA);
    await send(
      base,
      ix(
        "delegate",
        [
          { pubkey: payer.publicKey, isSigner: true, isWritable: true },
          { pubkey: buffer, isSigner: false, isWritable: true },
          { pubkey: record, isSigner: false, isWritable: true },
          { pubkey: metadata, isSigner: false, isWritable: true },
          { pubkey: COUNTER_PDA, isSigner: false, isWritable: true },
          { pubkey: PROGRAM_ID, isSigner: false, isWritable: false },
          { pubkey: DELEGATION_PROGRAM_ID, isSigner: false, isWritable: false },
          {
            pubkey: SystemProgram.programId,
            isSigner: false,
            isWritable: false,
          },
        ],
        Buffer.from(new PublicKey(ER_VALIDATOR_IDENTITY).toBytes()),
      ),
    );
    const delegated = await base.getAccountInfo(COUNTER_PDA);
    expect(delegated?.owner.toBase58()).toBe(DELEGATION_PROGRAM_ID.toBase58());

    // 4. increment on the ER (clones the account on first reference)
    const erCount0 = await waitFor(async () => await getCount(ephemeral));
    await send(
      ephemeral,
      ix("increment", [
        { pubkey: payer.publicKey, isSigner: true, isWritable: true },
        { pubkey: COUNTER_PDA, isSigner: false, isWritable: true },
      ]),
    );
    const erCount1 = await waitFor(async () => {
      const c = await getCount(ephemeral);
      return c === erCount0 + 1n ? c : false;
    });
    expect(erCount1).toBe(2n);

    // 5. commit ER state back to base
    const commitSig = await send(
      ephemeral,
      ix("commit", [
        { pubkey: payer.publicKey, isSigner: true, isWritable: true },
        { pubkey: COUNTER_PDA, isSigner: false, isWritable: true },
        { pubkey: MAGIC_PROGRAM_ID, isSigner: false, isWritable: false },
        { pubkey: MAGIC_CONTEXT_ID, isSigner: false, isWritable: true },
      ]),
    );
    await GetCommitmentSignature(commitSig, ephemeral);
    const baseAfterCommit = await waitFor(async () => {
      const c = await getCount(base);
      return c === 2n ? c : false;
    });
    expect(baseAfterCommit).toBe(2n);

    // 6. commit and undelegate
    const undelegateSig = await send(
      ephemeral,
      ix("commit_and_undelegate", [
        { pubkey: payer.publicKey, isSigner: true, isWritable: true },
        { pubkey: COUNTER_PDA, isSigner: false, isWritable: true },
        { pubkey: MAGIC_PROGRAM_ID, isSigner: false, isWritable: false },
        { pubkey: MAGIC_CONTEXT_ID, isSigner: false, isWritable: true },
      ]),
    );
    await GetCommitmentSignature(undelegateSig, ephemeral);
    const ownerBack = await waitFor(async () => {
      const acc = await base.getAccountInfo(COUNTER_PDA);
      return acc && acc.owner.equals(PROGRAM_ID) ? acc : false;
    });
    expect(ownerBack.owner.toBase58()).toBe(PROGRAM_ID.toBase58());
    expect(decodeCount(ownerBack.data)).toBe(2n);
  });
});
