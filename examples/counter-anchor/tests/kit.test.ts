// Counter lifecycle test using @solana/kit + @magicblock-labs/ephemeral-rollups-kit.
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
  signBytes,
  AccountRole,
  type Address,
  type IInstruction,
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
  COUNTER_SEED,
  PROGRAM_ID_BYTES,
  ER_VALIDATOR_IDENTITY,
  DELEGATION_PROGRAM_ID_STR,
  MAGIC_PROGRAM_ID_STR,
  MAGIC_CONTEXT_ID_STR,
  SYSTEM_PROGRAM_ID_STR,
  ixDiscriminator,
  decodeCount,
  waitFor,
} from "./_shared";

const addressEncoder = getAddressEncoder();
const base64 = getBase64Encoder();
const PROGRAM_ID = getAddressDecoder().decode(PROGRAM_ID_BYTES);
const DELEGATION_PROGRAM_ID = address(DELEGATION_PROGRAM_ID_STR);
const MAGIC_PROGRAM_ID = address(MAGIC_PROGRAM_ID_STR);
const MAGIC_CONTEXT_ID = address(MAGIC_CONTEXT_ID_STR);
const SYSTEM_PROGRAM_ID = address(SYSTEM_PROGRAM_ID_STR);

let base: Connection;
let ephemeral: Connection;
let signer: KeyPairSigner;
let COUNTER_PDA: Address;

function meta(addr: Address, role: AccountRole) {
  return { address: addr, role };
}

function ix(
  name: string,
  accounts: ReturnType<typeof meta>[],
  args = new Uint8Array(0),
): IInstruction {
  const disc = ixDiscriminator(name);
  const data = new Uint8Array(disc.length + args.length);
  data.set(disc, 0);
  data.set(args, disc.length);
  return { programAddress: PROGRAM_ID, accounts, data };
}

async function send(conn: Connection, instruction: IInstruction) {
  const { value: blockhash } = await conn.rpc.getLatestBlockhash().send();
  const tx = pipe(
    createTransactionMessage({ version: 0 }),
    (m) => setTransactionMessageFeePayerSigner(signer, m),
    (m) => setTransactionMessageLifetimeUsingBlockhash(blockhash, m),
    (m) => appendTransactionMessageInstruction(instruction, m),
  );
  // ER transactions use a non-delegated fee payer; skip preflight.
  const sig = await conn.sendTransaction(tx, [signer.keyPair], { skipPreflight: true });
  await conn.confirmTransaction(sig);
  return sig;
}

async function getCount(conn: Connection, addr: Address): Promise<bigint | null> {
  const { value } = await conn.rpc.getAccountInfo(addr, { encoding: "base64" }).send();
  if (!value) return null;
  const data = new Uint8Array(base64.encode(value.data[0]));
  return decodeCount(data);
}

async function getOwner(conn: Connection, addr: Address): Promise<string | null> {
  const { value } = await conn.rpc.getAccountInfo(addr, { encoding: "base64" }).send();
  return value ? value.owner : null;
}

describe("counter-anchor (kit)", () => {
  beforeAll(async () => {
    signer = await generateKeyPairSigner();
    [COUNTER_PDA] = await getProgramDerivedAddress({
      programAddress: PROGRAM_ID,
      seeds: [COUNTER_SEED, addressEncoder.encode(signer.address)],
    });

    base = await Connection.create(BASE_RPC_URL, BASE_WS_URL);
    const airdrop = await base.rpc
      .requestAirdrop(signer.address, lamports(5_000_000_000n))
      .send();
    await waitFor(async () => {
      const { value } = await base.rpc.getSignatureStatuses([airdrop]).send();
      const s = value[0];
      return s?.confirmationStatus === "confirmed" || s?.confirmationStatus === "finalized";
    });

    const { token } = await getAuthToken(ROUTER_RPC_URL, signer.address, async (msg) =>
      signBytes(signer.keyPair.privateKey, msg),
    );
    ephemeral = await Connection.create(
      `${ROUTER_RPC_URL}?token=${token}`,
      `${ROUTER_WS_URL}?token=${token}`,
    );
  });

  it("runs delegate -> increment-on-ER -> commit -> undelegate", async () => {
    // 1. initialize on base
    await send(
      base,
      ix("initialize", [
        meta(signer.address, AccountRole.WRITABLE_SIGNER),
        meta(COUNTER_PDA, AccountRole.WRITABLE),
        meta(SYSTEM_PROGRAM_ID, AccountRole.READONLY),
      ]),
    );
    expect(await getCount(base, COUNTER_PDA)).toBe(0n);

    // 2. increment on base
    await send(
      base,
      ix("increment", [
        meta(signer.address, AccountRole.WRITABLE_SIGNER),
        meta(COUNTER_PDA, AccountRole.WRITABLE),
      ]),
    );
    expect(await getCount(base, COUNTER_PDA)).toBe(1n);

    // 3. delegate the PDA
    const buffer = await delegateBufferPdaFromDelegatedAccountAndOwnerProgram(
      COUNTER_PDA,
      PROGRAM_ID,
    );
    const record = await delegationRecordPdaFromDelegatedAccount(COUNTER_PDA);
    const metadata = await delegationMetadataPdaFromDelegatedAccount(COUNTER_PDA);
    await send(
      base,
      ix(
        "delegate",
        [
          meta(signer.address, AccountRole.WRITABLE_SIGNER),
          meta(buffer, AccountRole.WRITABLE),
          meta(record, AccountRole.WRITABLE),
          meta(metadata, AccountRole.WRITABLE),
          meta(COUNTER_PDA, AccountRole.WRITABLE),
          meta(PROGRAM_ID, AccountRole.READONLY),
          meta(DELEGATION_PROGRAM_ID, AccountRole.READONLY),
          meta(SYSTEM_PROGRAM_ID, AccountRole.READONLY),
        ],
        new Uint8Array(addressEncoder.encode(address(ER_VALIDATOR_IDENTITY))),
      ),
    );
    expect(await getOwner(base, COUNTER_PDA)).toBe(DELEGATION_PROGRAM_ID);

    // 4. increment on the ER
    const erCount0 = await waitFor(async () => await getCount(ephemeral, COUNTER_PDA));
    await send(
      ephemeral,
      ix("increment", [
        meta(signer.address, AccountRole.WRITABLE_SIGNER),
        meta(COUNTER_PDA, AccountRole.WRITABLE),
      ]),
    );
    const erCount1 = await waitFor(async () => {
      const c = await getCount(ephemeral, COUNTER_PDA);
      return c === erCount0 + 1n ? c : false;
    });
    expect(erCount1).toBe(2n);

    // 5. commit ER state back to base
    await send(
      ephemeral,
      ix("commit", [
        meta(signer.address, AccountRole.WRITABLE_SIGNER),
        meta(COUNTER_PDA, AccountRole.WRITABLE),
        meta(MAGIC_PROGRAM_ID, AccountRole.READONLY),
        meta(MAGIC_CONTEXT_ID, AccountRole.WRITABLE),
      ]),
    );
    const baseAfterCommit = await waitFor(async () => {
      const c = await getCount(base, COUNTER_PDA);
      return c === 2n ? c : false;
    });
    expect(baseAfterCommit).toBe(2n);

    // 6. commit and undelegate
    await send(
      ephemeral,
      ix("commit_and_undelegate", [
        meta(signer.address, AccountRole.WRITABLE_SIGNER),
        meta(COUNTER_PDA, AccountRole.WRITABLE),
        meta(MAGIC_PROGRAM_ID, AccountRole.READONLY),
        meta(MAGIC_CONTEXT_ID, AccountRole.WRITABLE),
      ]),
    );
    const ownerBack = await waitFor(async () => {
      const owner = await getOwner(base, COUNTER_PDA);
      return owner === PROGRAM_ID ? owner : false;
    });
    expect(ownerBack).toBe(PROGRAM_ID);
    expect(await getCount(base, COUNTER_PDA)).toBe(2n);
  });
});
