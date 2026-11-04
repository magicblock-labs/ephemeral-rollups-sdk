import { afterEach, describe, expect, it, vi } from "vitest";
import {
  AccountMeta,
  AccountRole,
  address,
  compileTransactionMessage,
  createTransactionMessage,
  setTransactionMessageFeePayer,
} from "@solana/kit";
import { Connection } from "../connection";
import { getWritableAccounts } from "../utils";

const payer = address("11111111111111111111111111111111");
const writable = address("11111111111111111111111111111112");
const readonly = address("11111111111111111111111111111113");
const programAddress = address("MemoSq4gqABAXKb96qnH8TysNcWxMyWCqXgDLGmfcHr");

function transactionWithAccounts(accounts: readonly AccountMeta[] = []) {
  return setTransactionMessageFeePayer(payer, {
    ...createTransactionMessage({ version: 0 }),
    instructions: [{ programAddress, accounts }],
  });
}

afterEach(() => {
  vi.unstubAllGlobals();
});

describe("getWritableAccounts", () => {
  it("includes the fee payer when no instruction accounts are writable", () => {
    const transaction = transactionWithAccounts();
    const compiled = compileTransactionMessage(transaction);

    expect(compiled.staticAccounts[0]).toBe(payer);
    expect(compiled.header.numSignerAccounts).toBe(1);
    expect(compiled.header.numReadonlySignerAccounts).toBe(0);
    expect(getWritableAccounts(transaction)).toEqual([payer]);
  });

  it("includes a fee payer referenced as readonly and deduplicates writable accounts", () => {
    const transaction = transactionWithAccounts([
      { address: payer, role: AccountRole.READONLY_SIGNER },
      { address: writable, role: AccountRole.WRITABLE },
      { address: writable, role: AccountRole.WRITABLE_SIGNER },
      { address: readonly, role: AccountRole.READONLY },
    ]);

    expect(getWritableAccounts(transaction)).toEqual([payer, writable]);
  });

  it("does not duplicate the fee payer when instructions also make it writable", () => {
    const transaction = transactionWithAccounts([
      { address: payer, role: AccountRole.WRITABLE_SIGNER },
      { address: writable, role: AccountRole.WRITABLE },
    ]);

    expect(getWritableAccounts(transaction)).toEqual([payer, writable]);
  });

  it("continues to accept messages without a fee payer", () => {
    const transaction = {
      ...createTransactionMessage({ version: 0 }),
      instructions: [
        { programAddress },
        {
          programAddress,
          accounts: [
            { address: writable, role: AccountRole.WRITABLE },
            { address: readonly, role: AccountRole.READONLY_SIGNER },
          ],
        },
      ],
    };

    expect(getWritableAccounts(transaction)).toEqual([writable]);
    expect(
      getWritableAccounts(createTransactionMessage({ version: 0 })),
    ).toEqual([]);
  });
});

it("sends the fee payer to the router when preparing a payer-only writable transaction", async () => {
  const fetchMock = vi.fn(async () =>
    Response.json({
      jsonrpc: "2.0",
      id: 1,
      result: {
        blockhash: "11111111111111111111111111111111",
        lastValidBlockHeight: 100,
      },
    }),
  );
  vi.stubGlobal("fetch", fetchMock);

  const connection = await Connection.create("http://127.0.0.1:8899/");
  const blockhash = await connection.getLatestBlockhashForTransaction(
    transactionWithAccounts(),
  );

  expect(blockhash.lastValidBlockHeight).toBe(100n);
  expect(fetchMock).toHaveBeenCalledTimes(2);
  expect(fetchMock).toHaveBeenLastCalledWith("http://127.0.0.1:8899/", {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({
      jsonrpc: "2.0",
      id: 1,
      method: "getBlockhashForAccounts",
      params: [[payer]],
    }),
  });
});
