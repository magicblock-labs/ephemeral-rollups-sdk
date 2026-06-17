// Access-control (permission program) example using @solana/web3.js +
// @magicblock-labs/ephemeral-rollups-sdk.
//
// The access-control feature is an SDK instruction-builder API targeting the
// preloaded permission program (`ACLseoPoyC3cBqoUtkbjZ4aDrkurZW86v19pXz2XQnp1`), so
// this example is client-driven (no custom program). It creates, updates and closes
// a permission account.
import { describe, it, expect, beforeAll } from "vitest";
import { Connection, Keypair, Transaction, TransactionInstruction } from "@solana/web3.js";
import {
  PERMISSION_PROGRAM_ID,
  permissionPdaFromAccount,
  createCreatePermissionInstruction,
  createUpdatePermissionInstruction,
  createClosePermissionInstruction,
  AUTHORITY_FLAG,
} from "@magicblock-labs/ephemeral-rollups-sdk";

const BASE_RPC_URL = process.env.BASE_RPC_URL ?? "http://127.0.0.1:8899";
const base = new Connection(BASE_RPC_URL, "confirmed");
const payer = Keypair.generate();

async function send(instruction: TransactionInstruction) {
  const tx = new Transaction().add(instruction);
  const { blockhash, lastValidBlockHeight } = await base.getLatestBlockhash();
  tx.recentBlockhash = blockhash;
  tx.lastValidBlockHeight = lastValidBlockHeight;
  tx.feePayer = payer.publicKey;
  tx.sign(payer);
  const sig = await base.sendRawTransaction(tx.serialize());
  await base.confirmTransaction({ signature: sig, blockhash, lastValidBlockHeight }, "confirmed");
  return sig;
}

describe("access-control (web3.js)", () => {
  beforeAll(async () => {
    const sig = await base.requestAirdrop(payer.publicKey, 2_000_000_000);
    const bh = await base.getLatestBlockhash();
    await base.confirmTransaction({ signature: sig, ...bh }, "confirmed");
  });

  it("creates, updates and closes a permission", async () => {
    const permission = permissionPdaFromAccount(payer.publicKey);

    // create: the payer is the (sole) authority member
    await send(
      createCreatePermissionInstruction(
        { permissionedAccount: payer.publicKey, payer: payer.publicKey },
        { members: [{ pubkey: payer.publicKey, flags: AUTHORITY_FLAG }] },
      ),
    );
    const created = await base.getAccountInfo(permission);
    expect(created).not.toBeNull();
    expect(created!.owner.toBase58()).toBe(PERMISSION_PROGRAM_ID.toBase58());

    // update: add a second (non-authority) member
    const member = Keypair.generate().publicKey;
    await send(
      createUpdatePermissionInstruction(
        { authority: [payer.publicKey, true], permissionedAccount: [payer.publicKey, false] },
        {
          members: [
            { pubkey: payer.publicKey, flags: AUTHORITY_FLAG },
            { pubkey: member, flags: 0 },
          ],
        },
      ),
    );
    expect(await base.getAccountInfo(permission)).not.toBeNull();

    // close: the permission account is removed
    await send(
      createClosePermissionInstruction({
        payer: payer.publicKey,
        authority: [payer.publicKey, true],
        permissionedAccount: [payer.publicKey, false],
      }),
    );
    expect(await base.getAccountInfo(permission)).toBeNull();
  });
});
