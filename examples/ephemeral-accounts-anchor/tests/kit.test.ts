// #[ephemeral_accounts] example using @solana/kit + ephemeral-rollups-kit.
import { describe, it, expect, beforeAll } from "vitest";
import {
  address,
  appendTransactionMessageInstruction,
  createTransactionMessage,
  generateKeyPairSigner,
  getAddressEncoder,
  getAddressDecoder,
  getProgramDerivedAddress,
  lamports,
  pipe,
  setTransactionMessageFeePayerSigner,
  setTransactionMessageLifetimeUsingBlockhash,
  AccountRole,
  type Address,
  type Instruction,
  type KeyPairSigner,
} from "@solana/kit";
import {
  Connection,
  getAuthToken,
  delegationRecordPdaFromDelegatedAccount,
  delegationMetadataPdaFromDelegatedAccount,
  delegateBufferPdaFromDelegatedAccountAndOwnerProgram,
} from "@magicblock-labs/ephemeral-rollups-kit";
import {
  BASE_RPC_URL,
  BASE_WS_URL,
  ROUTER_RPC_URL,
  ROUTER_WS_URL,
  ER_VALIDATOR_IDENTITY,
  TREASURY_SEED,
  GAME_SEED,
  PROGRAM_ID_BYTES,
  EPHEMERAL_VAULT_ID_STR,
  MAGIC_PROGRAM_ID_STR,
  SYSTEM_PROGRAM_ID_STR,
  ixDiscriminator,
  waitFor,
} from "./_shared";

const addressEncoder = getAddressEncoder();
const PROGRAM_ID = getAddressDecoder().decode(PROGRAM_ID_BYTES);
const DELEGATION_PROGRAM_ID = address(
  "DELeGGvXpWV2fqJUhqcF5ZSYMS4JTLjteaAMARRSaeSh",
);
const VAULT = address(EPHEMERAL_VAULT_ID_STR);
const MAGIC_PROGRAM_ID = address(MAGIC_PROGRAM_ID_STR);
const SYSTEM_PROGRAM_ID = address(SYSTEM_PROGRAM_ID_STR);

let base: Connection;
let ephemeral: Connection;
let payer: KeyPairSigner;
let TREASURY: Address;
let GAME: Address;

function meta(addr: Address, role: AccountRole) {
  return { address: addr, role };
}

async function send(conn: Connection, ix: Instruction) {
  const { value: blockhash } = await conn.rpc.getLatestBlockhash().send();
  const tx = pipe(
    createTransactionMessage({ version: 0 }),
    (m) => setTransactionMessageFeePayerSigner(payer, m),
    (m) => setTransactionMessageLifetimeUsingBlockhash(blockhash, m),
    (m) => appendTransactionMessageInstruction(ix, m),
  );
  const sig = await conn.sendTransaction(tx, [payer.keyPair], {
    skipPreflight: true,
  });
  await waitFor(async () => {
    const { value } = await conn.rpc.getSignatureStatuses([sig]).send();
    const s = value[0];
    if (!s) return false;
    if (s.err)
      throw new Error(`transaction ${sig} failed: ${JSON.stringify(s.err)}`);
    return (
      s.confirmationStatus === "confirmed" ||
      s.confirmationStatus === "finalized"
    );
  });
  return sig;
}

const u32 = (n: number) => {
  const b = new Uint8Array(4);
  new DataView(b.buffer).setUint32(0, n, true);
  return b;
};
const u64 = (n: bigint) => {
  const b = new Uint8Array(8);
  new DataView(b.buffer).setBigUint64(0, n, true);
  return b;
};
const concat = (...parts: Uint8Array[]) => {
  const out = new Uint8Array(parts.reduce((n, p) => n + p.length, 0));
  let o = 0;
  for (const p of parts) {
    out.set(p, o);
    o += p.length;
  }
  return out;
};

describe("ephemeral-accounts-anchor (kit)", () => {
  beforeAll(async () => {
    payer = await generateKeyPairSigner();
    [TREASURY] = await getProgramDerivedAddress({
      programAddress: PROGRAM_ID,
      seeds: [TREASURY_SEED, addressEncoder.encode(payer.address)],
    });
    [GAME] = await getProgramDerivedAddress({
      programAddress: PROGRAM_ID,
      seeds: [GAME_SEED, addressEncoder.encode(payer.address)],
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
    const { token } = await getAuthToken(
      ROUTER_RPC_URL,
      payer.address,
      async (msg) =>
        (await import("@solana/kit")).signBytes(payer.keyPair.privateKey, msg),
    );
    ephemeral = await Connection.create(
      `${ROUTER_RPC_URL}?token=${token}`,
      `${ROUTER_WS_URL}?token=${token}`,
    );
  });

  it("delegates a sponsor treasury and creates a gas-sponsored ephemeral account on the ER", async () => {
    await send(base, {
      programAddress: PROGRAM_ID,
      accounts: [
        meta(payer.address, AccountRole.WRITABLE_SIGNER),
        meta(TREASURY, AccountRole.WRITABLE),
        meta(SYSTEM_PROGRAM_ID, AccountRole.READONLY),
      ],
      data: concat(
        new Uint8Array(ixDiscriminator("init_treasury")),
        u64(100_000_000n),
      ),
    });

    const buffer = await delegateBufferPdaFromDelegatedAccountAndOwnerProgram(
      TREASURY,
      PROGRAM_ID,
    );
    const record = await delegationRecordPdaFromDelegatedAccount(TREASURY);
    const metadata = await delegationMetadataPdaFromDelegatedAccount(TREASURY);
    await send(base, {
      programAddress: PROGRAM_ID,
      accounts: [
        meta(payer.address, AccountRole.WRITABLE_SIGNER),
        meta(buffer, AccountRole.WRITABLE),
        meta(record, AccountRole.WRITABLE),
        meta(metadata, AccountRole.WRITABLE),
        meta(TREASURY, AccountRole.WRITABLE),
        meta(PROGRAM_ID, AccountRole.READONLY),
        meta(DELEGATION_PROGRAM_ID, AccountRole.READONLY),
        meta(SYSTEM_PROGRAM_ID, AccountRole.READONLY),
      ],
      data: concat(
        new Uint8Array(ixDiscriminator("delegate_treasury")),
        new Uint8Array(addressEncoder.encode(address(ER_VALIDATOR_IDENTITY))),
      ),
    });

    await waitFor(async () => {
      const { value } = await ephemeral.rpc
        .getAccountInfo(TREASURY, { encoding: "base64" })
        .send();
      return value !== null;
    });

    const SIZE = 32;
    await send(ephemeral, {
      programAddress: PROGRAM_ID,
      accounts: [
        meta(payer.address, AccountRole.WRITABLE_SIGNER), // authority + fee payer
        meta(TREASURY, AccountRole.WRITABLE),
        meta(GAME, AccountRole.WRITABLE),
        meta(VAULT, AccountRole.WRITABLE),
        meta(MAGIC_PROGRAM_ID, AccountRole.READONLY),
      ],
      data: concat(new Uint8Array(ixDiscriminator("create_game")), u32(SIZE)),
    });

    const game = await waitFor(async () => {
      const { value } = await ephemeral.rpc
        .getAccountInfo(GAME, { encoding: "base64" })
        .send();
      return value ?? false;
    });
    expect(Buffer.from(game.data[0], "base64").length).toBe(SIZE);
    expect(game.owner).toBe(PROGRAM_ID);
  });
});
