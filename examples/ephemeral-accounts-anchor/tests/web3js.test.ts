// #[ephemeral_accounts] example using @solana/web3.js + ephemeral-rollups-sdk.
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
  ER_VALIDATOR_IDENTITY,
  TREASURY_SEED,
  GAME_SEED,
  PROGRAM_ID_BYTES,
  EPHEMERAL_VAULT_ID_STR,
  MAGIC_PROGRAM_ID_STR,
  ixDiscriminator,
  waitFor,
} from "./_shared";

const PROGRAM_ID = new PublicKey(PROGRAM_ID_BYTES);
const VAULT = new PublicKey(EPHEMERAL_VAULT_ID_STR);
const MAGIC_PROGRAM_ID = new PublicKey(MAGIC_PROGRAM_ID_STR);

const base = new Connection(BASE_RPC_URL, "confirmed");
let ephemeral: Connection;
const payer = Keypair.generate();
const [TREASURY] = PublicKey.findProgramAddressSync(
  [TREASURY_SEED, payer.publicKey.toBytes()],
  PROGRAM_ID,
);
const [GAME] = PublicKey.findProgramAddressSync(
  [GAME_SEED, payer.publicKey.toBytes()],
  PROGRAM_ID,
);

async function send(
  conn: Connection,
  ix: TransactionInstruction,
  skipPreflight = false,
) {
  const tx = new Transaction().add(ix);
  const { blockhash, lastValidBlockHeight } = await conn.getLatestBlockhash();
  tx.recentBlockhash = blockhash;
  tx.lastValidBlockHeight = lastValidBlockHeight;
  tx.feePayer = payer.publicKey;
  tx.sign(payer);
  const sig = await conn.sendRawTransaction(tx.serialize(), { skipPreflight });
  await conn.confirmTransaction(
    { signature: sig, blockhash, lastValidBlockHeight },
    "confirmed",
  );
  return sig;
}

const u32 = (n: number) => {
  const b = Buffer.alloc(4);
  b.writeUInt32LE(n);
  return b;
};
const u64 = (n: bigint) => {
  const b = Buffer.alloc(8);
  b.writeBigUInt64LE(n);
  return b;
};

describe("ephemeral-accounts-anchor (web3.js)", () => {
  beforeAll(async () => {
    const sig = await base.requestAirdrop(payer.publicKey, 5_000_000_000);
    const bh = await base.getLatestBlockhash();
    await base.confirmTransaction({ signature: sig, ...bh }, "confirmed");

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

  it("delegates a sponsor treasury and creates a gas-sponsored ephemeral account on the ER", async () => {
    // 1. create + fund the sponsor treasury on base
    await send(
      base,
      new TransactionInstruction({
        programId: PROGRAM_ID,
        keys: [
          { pubkey: payer.publicKey, isSigner: true, isWritable: true },
          { pubkey: TREASURY, isSigner: false, isWritable: true },
          {
            pubkey: SystemProgram.programId,
            isSigner: false,
            isWritable: false,
          },
        ],
        data: Buffer.concat([
          ixDiscriminator("init_treasury"),
          u64(100_000_000n),
        ]),
      }),
    );

    // 2. delegate the treasury to the ER
    const buffer = delegateBufferPdaFromDelegatedAccountAndOwnerProgram(
      TREASURY,
      PROGRAM_ID,
    );
    const record = delegationRecordPdaFromDelegatedAccount(TREASURY);
    const metadata = delegationMetadataPdaFromDelegatedAccount(TREASURY);
    await send(
      base,
      new TransactionInstruction({
        programId: PROGRAM_ID,
        keys: [
          { pubkey: payer.publicKey, isSigner: true, isWritable: true },
          { pubkey: buffer, isSigner: false, isWritable: true },
          { pubkey: record, isSigner: false, isWritable: true },
          { pubkey: metadata, isSigner: false, isWritable: true },
          { pubkey: TREASURY, isSigner: false, isWritable: true },
          { pubkey: PROGRAM_ID, isSigner: false, isWritable: false },
          { pubkey: DELEGATION_PROGRAM_ID, isSigner: false, isWritable: false },
          {
            pubkey: SystemProgram.programId,
            isSigner: false,
            isWritable: false,
          },
        ],
        data: Buffer.concat([
          ixDiscriminator("delegate_treasury"),
          Buffer.from(new PublicKey(ER_VALIDATOR_IDENTITY).toBytes()),
        ]),
      }),
    );
    expect((await base.getAccountInfo(TREASURY))?.owner.toBase58()).toBe(
      DELEGATION_PROGRAM_ID.toBase58(),
    );

    // 3. create the gas-sponsored ephemeral account on the ER (fee payer = wallet,
    //    sponsor = delegated treasury)
    const SIZE = 32;
    await send(
      ephemeral,
      new TransactionInstruction({
        programId: PROGRAM_ID,
        keys: [
          { pubkey: payer.publicKey, isSigner: true, isWritable: true }, // authority + fee payer
          { pubkey: TREASURY, isSigner: false, isWritable: true },
          { pubkey: GAME, isSigner: false, isWritable: true },
          { pubkey: VAULT, isSigner: false, isWritable: true },
          { pubkey: MAGIC_PROGRAM_ID, isSigner: false, isWritable: false },
        ],
        data: Buffer.concat([ixDiscriminator("create_game"), u32(SIZE)]),
      }),
      true,
    );

    const game = await waitFor(
      async () => await ephemeral.getAccountInfo(GAME),
    );
    expect(game.data.length).toBe(SIZE);
    expect(game.owner.toBase58()).toBe(PROGRAM_ID.toBase58());
  });
});
