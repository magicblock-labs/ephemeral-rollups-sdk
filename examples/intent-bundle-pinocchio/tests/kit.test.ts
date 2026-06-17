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
  TAG,
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
let COUNTER_BUMP: number;

function meta(addr: Address, role: AccountRole) {
  return { address: addr, role };
}

function ix(accounts: ReturnType<typeof meta>[], data: Uint8Array): IInstruction {
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
  const sig = await conn.sendTransaction(tx, [signer.keyPair], { skipPreflight: true });
  await conn.confirmTransaction(sig);
  return sig;
}

async function getCount(conn: Connection): Promise<bigint | null> {
  const { value } = await conn.rpc.getAccountInfo(COUNTER_PDA, { encoding: "base64" }).send();
  if (!value) return null;
  return decodeCount(new Uint8Array(base64.encode(value.data[0])));
}

async function getOwner(conn: Connection): Promise<string | null> {
  const { value } = await conn.rpc.getAccountInfo(COUNTER_PDA, { encoding: "base64" }).send();
  return value ? value.owner : null;
}

function payerMeta() {
  return meta(signer.address, AccountRole.WRITABLE_SIGNER);
}
function counterMeta() {
  return meta(COUNTER_PDA, AccountRole.WRITABLE);
}

describe("intent-bundle-pinocchio (kit)", () => {
  beforeAll(async () => {
    signer = await generateKeyPairSigner();
    [COUNTER_PDA, COUNTER_BUMP] = await getProgramDerivedAddress({
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
      ix(
        [payerMeta(), counterMeta(), meta(SYSTEM_PROGRAM_ID, AccountRole.READONLY)],
        new Uint8Array([TAG.initialize, COUNTER_BUMP]),
      ),
    );
    expect(await getCount(base)).toBe(0n);

    // 2. increment on base
    await send(base, ix([payerMeta(), counterMeta()], new Uint8Array([TAG.increment])));
    expect(await getCount(base)).toBe(1n);

    // 3. delegate the PDA
    const buffer = await delegateBufferPdaFromDelegatedAccountAndOwnerProgram(COUNTER_PDA, PROGRAM_ID);
    const record = await delegationRecordPdaFromDelegatedAccount(COUNTER_PDA);
    const metadata = await delegationMetadataPdaFromDelegatedAccount(COUNTER_PDA);
    const delegateData = new Uint8Array(2 + 32);
    delegateData[0] = TAG.delegate;
    delegateData[1] = COUNTER_BUMP;
    delegateData.set(addressEncoder.encode(address(ER_VALIDATOR_IDENTITY)), 2);
    await send(
      base,
      ix(
        [
          payerMeta(),
          counterMeta(),
          meta(PROGRAM_ID, AccountRole.READONLY),
          meta(buffer, AccountRole.WRITABLE),
          meta(record, AccountRole.WRITABLE),
          meta(metadata, AccountRole.WRITABLE),
          meta(SYSTEM_PROGRAM_ID, AccountRole.READONLY),
          meta(DELEGATION_PROGRAM_ID, AccountRole.READONLY),
        ],
        delegateData,
      ),
    );
    expect(await getOwner(base)).toBe(DELEGATION_PROGRAM_ID);

    // 4. increment on the ER
    const erCount0 = await waitFor(async () => await getCount(ephemeral));
    await send(ephemeral, ix([payerMeta(), counterMeta()], new Uint8Array([TAG.increment])));
    const erCount1 = await waitFor(async () => {
      const c = await getCount(ephemeral);
      return c === erCount0 + 1n ? c : false;
    });
    expect(erCount1).toBe(2n);

    // 5. commit ER state back to base
    await send(
      ephemeral,
      ix(
        [
          payerMeta(),
          counterMeta(),
          meta(MAGIC_PROGRAM_ID, AccountRole.READONLY),
          meta(MAGIC_CONTEXT_ID, AccountRole.WRITABLE),
        ],
        new Uint8Array([TAG.commit]),
      ),
    );
    const baseAfterCommit = await waitFor(async () => {
      const c = await getCount(base);
      return c === 2n ? c : false;
    });
    expect(baseAfterCommit).toBe(2n);

    // 6. commit and undelegate
    await send(
      ephemeral,
      ix(
        [
          payerMeta(),
          counterMeta(),
          meta(MAGIC_PROGRAM_ID, AccountRole.READONLY),
          meta(MAGIC_CONTEXT_ID, AccountRole.WRITABLE),
        ],
        new Uint8Array([TAG.commitAndUndelegate]),
      ),
    );
    const ownerBack = await waitFor(async () => {
      const owner = await getOwner(base);
      return owner === PROGRAM_ID ? owner : false;
    });
    expect(ownerBack).toBe(PROGRAM_ID);
    expect(await getCount(base)).toBe(2n);
  });
});
