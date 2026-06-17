// Ephemeral SPL token example using @solana/web3.js + @magicblock-labs/ephemeral-rollups-sdk.
//
// Demonstrates the ephemeral-ATA flow against the preloaded ephemeral SPL token program
// (`SPLxh1LVZzEkX99H6rqYizhytLWPZVV296zyYDPagv2`): set up a mint, initialize the global
// vault, create an ephemeral ATA, and deposit SPL tokens into it so they can be used on
// the rollup. Client-driven (no custom program).
import { describe, it, expect, beforeAll } from "vitest";
import { Connection, Keypair, Transaction, TransactionInstruction } from "@solana/web3.js";
import {
  createMint,
  getOrCreateAssociatedTokenAccount,
  mintTo,
  TOKEN_PROGRAM_ID,
} from "@solana/spl-token";
import {
  deriveVault,
  deriveVaultAta,
  deriveRentPda,
  deriveEphemeralAta,
  initVaultIx,
  initVaultAtaIx,
  initRentPdaIx,
  initEphemeralAtaIx,
  depositSplTokensIx,
  decodeEphemeralAta,
} from "@magicblock-labs/ephemeral-rollups-sdk";

const BASE_RPC_URL = process.env.BASE_RPC_URL ?? "http://127.0.0.1:8899";
const base = new Connection(BASE_RPC_URL, "confirmed");
const payer = Keypair.generate();

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

describe("spl (web3.js)", () => {
  beforeAll(async () => {
    const sig = await base.requestAirdrop(payer.publicKey, 5_000_000_000);
    const bh = await base.getLatestBlockhash();
    await base.confirmTransaction({ signature: sig, ...bh }, "confirmed");
  });

  it("initializes a vault + ephemeral ATA and deposits SPL tokens", async () => {
    // mint setup
    const mint = await createMint(base, payer, payer.publicKey, null, 0);
    const sourceAta = (
      await getOrCreateAssociatedTokenAccount(base, payer, mint, payer.publicKey)
    ).address;
    await mintTo(base, payer, mint, sourceAta, payer, 1000);

    const [vault] = deriveVault(mint);
    const vaultAta = deriveVaultAta(mint, vault);
    const [rentPda] = deriveRentPda();
    const [ephemeralAta] = deriveEphemeralAta(payer.publicKey, mint);

    // initialize vault, vault ATA, rent PDA and the ephemeral ATA
    await send([
      initVaultIx(vault, mint, payer.publicKey),
      initVaultAtaIx(payer.publicKey, vaultAta, vault, mint),
      initRentPdaIx(payer.publicKey, rentPda),
      initEphemeralAtaIx(ephemeralAta, payer.publicKey, mint, payer.publicKey),
    ]);

    // deposit 250 tokens into the ephemeral ATA
    await send([
      depositSplTokensIx(
        ephemeralAta,
        vault,
        mint,
        sourceAta,
        vaultAta,
        payer.publicKey,
        250n,
        TOKEN_PROGRAM_ID,
      ),
    ]);

    const eata = decodeEphemeralAta((await base.getAccountInfo(ephemeralAta))!);
    expect(eata.mint.toBase58()).toBe(mint.toBase58());
    expect(eata.owner.toBase58()).toBe(payer.publicKey.toBase58());
    expect(eata.amount).toBe(250n);
  });
});
