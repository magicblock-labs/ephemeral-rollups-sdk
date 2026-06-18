// Access-control (permission program) example using @solana/kit +
// @magicblock-labs/ephemeral-rollups-kit.
import { describe, it, expect, beforeAll } from "vitest";
import {
  appendTransactionMessageInstruction,
  createTransactionMessage,
  generateKeyPairSigner,
  lamports,
  pipe,
  setTransactionMessageFeePayerSigner,
  setTransactionMessageLifetimeUsingBlockhash,
  type Instruction,
  type KeyPairSigner,
} from "@solana/kit";
import {
  Connection,
  PERMISSION_PROGRAM_ID,
  permissionPdaFromAccount,
  createCreatePermissionInstruction,
  createUpdatePermissionInstruction,
  createClosePermissionInstruction,
  AUTHORITY_FLAG,
} from "@magicblock-labs/ephemeral-rollups-kit";

const BASE_RPC_URL = process.env.BASE_RPC_URL ?? "http://127.0.0.1:8899";
const BASE_WS_URL = process.env.BASE_WS_URL ?? "ws://127.0.0.1:8900";

let base: Connection;
let payer: KeyPairSigner;

async function send(instruction: Instruction) {
  const { value: blockhash } = await base.rpc.getLatestBlockhash().send();
  const tx = pipe(
    createTransactionMessage({ version: 0 }),
    (m) => setTransactionMessageFeePayerSigner(payer, m),
    (m) => setTransactionMessageLifetimeUsingBlockhash(blockhash, m),
    (m) => appendTransactionMessageInstruction(instruction, m),
  );
  const sig = await base.sendTransaction(tx, [payer.keyPair]);
  for (;;) {
    const { value } = await base.rpc.getSignatureStatuses([sig]).send();
    const s = value[0];
    if (s?.confirmationStatus === "confirmed" || s?.confirmationStatus === "finalized") break;
    await new Promise((r) => setTimeout(r, 500));
  }
  return sig;
}

async function exists(addr: Awaited<ReturnType<typeof permissionPdaFromAccount>>) {
  const { value } = await base.rpc.getAccountInfo(addr, { encoding: "base64" }).send();
  return value;
}

describe("access-control (kit)", () => {
  beforeAll(async () => {
    payer = await generateKeyPairSigner();
    base = await Connection.create(BASE_RPC_URL, BASE_WS_URL);
    const airdrop = await base.rpc.requestAirdrop(payer.address, lamports(2_000_000_000n)).send();
    for (;;) {
      const { value } = await base.rpc.getSignatureStatuses([airdrop]).send();
      const s = value[0];
      if (s?.confirmationStatus === "confirmed" || s?.confirmationStatus === "finalized") break;
      await new Promise((r) => setTimeout(r, 500));
    }
  });

  it("creates, updates and closes a permission", async () => {
    const permission = await permissionPdaFromAccount(payer.address);

    await send(
      await createCreatePermissionInstruction(
        { permissionedAccount: payer.address, payer: payer.address },
        { members: [{ pubkey: payer.address, flags: AUTHORITY_FLAG }] },
      ),
    );
    const created = await exists(permission);
    expect(created).not.toBeNull();
    expect(created!.owner).toBe(PERMISSION_PROGRAM_ID);

    const member = (await generateKeyPairSigner()).address;
    await send(
      await createUpdatePermissionInstruction(
        { authority: [payer.address, true], permissionedAccount: [payer.address, false] },
        {
          members: [
            { pubkey: payer.address, flags: AUTHORITY_FLAG },
            { pubkey: member, flags: 0 },
          ],
        },
      ),
    );
    expect(await exists(permission)).not.toBeNull();

    await send(
      await createClosePermissionInstruction({
        payer: payer.address,
        authority: [payer.address, true],
        permissionedAccount: [payer.address, false],
      }),
    );
    expect(await exists(permission)).toBeNull();
  });
});
