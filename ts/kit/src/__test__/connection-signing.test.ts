import { afterEach, expect, it, vi } from "vitest";
import {
  AccountRole,
  address,
  appendTransactionMessageInstruction,
  blockhash,
  compileTransaction,
  createTransactionMessage,
  generateKeyPair,
  getAddressFromPublicKey,
  getBase64EncodedWireTransaction,
  isFullySignedTransaction,
  partiallySignTransaction,
  pipe,
  setTransactionMessageFeePayer,
  setTransactionMessageLifetimeUsingBlockhash,
} from "@solana/kit";
import { Connection } from "../index";

afterEach(() => vi.unstubAllGlobals());

it.each(["message", "unsigned", "partial", "presigned"])(
  "applies supplied signers to a %s transaction before sending",
  async (mode) => {
    const payer = await generateKeyPair();
    const other = await generateKeyPair();
    const payerAddress = await getAddressFromPublicKey(payer.publicKey);
    const otherAddress = await getAddressFromPublicKey(other.publicKey);
    const message = pipe(
      createTransactionMessage({ version: 0 }),
      (tx) => setTransactionMessageFeePayer(payerAddress, tx),
      (tx) =>
        setTransactionMessageLifetimeUsingBlockhash(
          {
            blockhash: blockhash("11111111111111111111111111111111"),
            lastValidBlockHeight: 100n,
          },
          tx,
        ),
      (tx) =>
        appendTransactionMessageInstruction(
          {
            programAddress: address(
              "MemoSq4gqABAXKb96qnH8TysNcWxMyWCqXgDLGmfcHr",
            ),
            accounts: [
              { address: otherAddress, role: AccountRole.READONLY_SIGNER },
            ],
            data: new TextEncoder().encode("signing regression"),
          },
          tx,
        ),
    );
    const compiled = compileTransaction(message);
    const partial = await partiallySignTransaction([payer], compiled);
    const signed = await partiallySignTransaction([payer, other], compiled);
    expect(isFullySignedTransaction(compiled)).toBe(false);
    expect(isFullySignedTransaction(partial)).toBe(false);
    expect(isFullySignedTransaction(signed)).toBe(true);
    const compiledBefore = structuredClone(compiled);
    const partialBefore = structuredClone(partial);

    const requests: string[] = [];
    let sentWire: unknown;
    vi.stubGlobal("fetch", async (url: string, init: RequestInit) => {
      expect(String(url)).toBe("http://127.0.0.1:8899/");
      const request: unknown = JSON.parse(String(init.body));
      if (
        typeof request !== "object" ||
        request === null ||
        !("method" in request) ||
        !("id" in request) ||
        !("params" in request) ||
        !Array.isArray(request.params) ||
        typeof request.method !== "string"
      ) {
        throw new Error("Unexpected JSON-RPC request");
      }
      requests.push(request.method);
      if (request.method === "getBlockhashForAccounts") {
        return Response.json({
          jsonrpc: "2.0",
          id: request.id,
          error: { code: -32601, message: "Method not found" },
        });
      }
      expect(request.method).toBe("sendTransaction");
      sentWire = request.params[0];
      return Response.json({
        jsonrpc: "2.0",
        id: request.id,
        result: "1".repeat(64),
      });
    });
    const connection = await Connection.create("http://127.0.0.1:8899/");
    const input =
      mode === "message"
        ? message
        : mode === "partial"
          ? partial
          : mode === "presigned"
            ? signed
            : compiled;
    const signers =
      mode === "presigned" ? [] : mode === "partial" ? [other] : [payer, other];
    await connection.sendTransaction(input, signers);
    expect(requests).toEqual(["getBlockhashForAccounts", "sendTransaction"]);
    expect(sentWire).toBe(getBase64EncodedWireTransaction(signed));
    // Signing must not mutate either caller-owned compiled input.
    expect(compiled).toEqual(compiledBefore);
    expect(partial).toEqual(partialBefore);
    expect(isFullySignedTransaction(compiled)).toBe(false);
    expect(isFullySignedTransaction(partial)).toBe(false);
  },
);
