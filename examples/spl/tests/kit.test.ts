// Ephemeral SPL token example using @solana/kit + @magicblock-labs/ephemeral-rollups-kit.
//
// The SPL *mint* is set up with @solana/spl-token (web3.js) because there is no
// @solana/kit v4-compatible SPL-token client; the ephemeral-ATA feature under test is
// exercised entirely through the kit SDK.
import { describe, it, expect, beforeAll } from "vitest";
import {
  Connection as Web3Connection,
  Keypair,
} from "@solana/web3.js";
import {
  createMint,
  getOrCreateAssociatedTokenAccount,
  mintTo,
} from "@solana/spl-token";
import {
  address,
  appendTransactionMessageInstructions,
  createKeyPairSignerFromBytes,
  createTransactionMessage,
  pipe,
  setTransactionMessageFeePayerSigner,
  setTransactionMessageLifetimeUsingBlockhash,
  type Address,
  type Instruction,
  type KeyPairSigner,
} from "@solana/kit";
import {
  Connection,
  deriveVault,
  deriveVaultAta,
  deriveRentPda,
  deriveEphemeralAta,
  initVaultIx,
  initVaultAtaIx,
  initRentPdaIx,
  initEphemeralAtaIx,
  transferToVaultIx,
  decodeEphemeralAta,
} from "@magicblock-labs/ephemeral-rollups-kit";

const BASE_RPC_URL = process.env.BASE_RPC_URL ?? "http://127.0.0.1:8899";
const BASE_WS_URL = process.env.BASE_WS_URL ?? "ws://127.0.0.1:8900";

const web3 = new Web3Connection(BASE_RPC_URL, "confirmed");
const payerKp = Keypair.generate();
let kit: Connection;
let payer: KeyPairSigner;

async function send(ixs: Instruction[]) {
  const { value: blockhash } = await kit.rpc.getLatestBlockhash().send();
  const tx = pipe(
    createTransactionMessage({ version: 0 }),
    (m) => setTransactionMessageFeePayerSigner(payer, m),
    (m) => setTransactionMessageLifetimeUsingBlockhash(blockhash, m),
    (m) => appendTransactionMessageInstructions(ixs, m),
  );
  const sig = await kit.sendTransaction(tx, [payer.keyPair]);
  for (;;) {
    const { value } = await kit.rpc.getSignatureStatuses([sig]).send();
    const s = value[0];
    if (s?.confirmationStatus === "confirmed" || s?.confirmationStatus === "finalized") break;
    await new Promise((r) => setTimeout(r, 500));
  }
  return sig;
}

describe("spl (kit)", () => {
  beforeAll(async () => {
    kit = await Connection.create(BASE_RPC_URL, BASE_WS_URL);
    payer = await createKeyPairSignerFromBytes(payerKp.secretKey);
    const sig = await web3.requestAirdrop(payerKp.publicKey, 5_000_000_000);
    const bh = await web3.getLatestBlockhash();
    await web3.confirmTransaction({ signature: sig, ...bh }, "confirmed");
  });

  it("initializes a vault + ephemeral ATA and deposits SPL tokens", async () => {
    // mint setup via @solana/spl-token
    const mintPk = await createMint(web3, payerKp, payerKp.publicKey, null, 0);
    const sourceAtaPk = (
      await getOrCreateAssociatedTokenAccount(web3, payerKp, mintPk, payerKp.publicKey)
    ).address;
    await mintTo(web3, payerKp, mintPk, sourceAtaPk, payerKp, 1000);

    const mint = address(mintPk.toBase58());
    const sourceAta = address(sourceAtaPk.toBase58());
    const [vault] = await deriveVault(mint);
    const vaultAta = await deriveVaultAta(mint, vault);
    const [vaultEphemeralAta] = await deriveEphemeralAta(vault, mint);
    const [rentPda] = await deriveRentPda();
    const [ephemeralAta] = await deriveEphemeralAta(payer.address, mint);

    await send([
      initVaultIx(vault, mint, payer.address, vaultEphemeralAta, vaultAta),
      initVaultAtaIx(payer.address, vaultAta, vault, mint),
      initRentPdaIx(payer.address, rentPda),
      initEphemeralAtaIx(ephemeralAta, payer.address, mint, payer.address),
    ]);

    await send([
      transferToVaultIx(ephemeralAta, vault, mint, sourceAta, vaultAta, payer.address, 250n),
    ]);

    const { value } = await kit.rpc
      .getAccountInfo(ephemeralAta as Address, { encoding: "base64" })
      .send();
    const eata = decodeEphemeralAta(value!);
    expect(eata.amount).toBe(250n);
  });
});
