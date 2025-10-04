import { Connection, getWritableAccounts } from "../magic-router.js";
import {
  Transaction,
  Keypair,
  PublicKey,
  SendTransactionError,
} from "@solana/web3.js";

// --- Global mocks ---
global.fetch = jest.fn();

describe("getWritableAccounts", () => {
  const mockPublicKey = (address: string) => ({
    toBase58: () => address,
    toString: () => address,
  });

  it("deduplicates writable accounts", () => {
    const tx = {
      feePayer: mockPublicKey("fee-payer"),
      instructions: [
        {
          keys: [
            { pubkey: mockPublicKey("k1"), isWritable: true },
            { pubkey: mockPublicKey("k1"), isWritable: true },
          ],
        },
      ],
    } as unknown as Transaction;

    const result = getWritableAccounts(tx);
    expect(result).toEqual(["fee-payer", "k1"]); // ✅ no extra accounts
  });
});

describe("Connection prototype methods", () => {
  let connection: Connection;
  let tx: Transaction;

  beforeEach(() => {
    connection = new Connection("http://localhost"); // use patched Connection
    tx = new Transaction();

    // Mock transaction instance methods
    (tx as any).serialize = jest.fn(() => Buffer.from("mock"));
    (tx as any).sign = jest.fn();

    (global.fetch as jest.Mock).mockReset();
  });

  it("getClosestValidator returns identity", async () => {
    (global.fetch as jest.Mock).mockResolvedValueOnce({
      json: async () => ({ result: { identity: "validator-1" } }),
    });

    const result = await (connection as any).getClosestValidator();
    expect(result).toEqual({ identity: "validator-1" });
  });

  it("getDelegationStatus works with string account", async () => {
    (global.fetch as jest.Mock).mockResolvedValueOnce({
      json: async () => ({ result: { isDelegated: true } }),
    });

    const result = await (connection as any).getDelegationStatus("account1");
    expect(result).toEqual({ isDelegated: true });
  });

  it("getDelegationStatus works with PublicKey account", async () => {
    (global.fetch as jest.Mock).mockResolvedValueOnce({
      json: async () => ({ result: { isDelegated: false } }),
    });

    const pk = new PublicKey("11111111111111111111111111111111");
    const result = await (connection as any).getDelegationStatus(pk);
    expect(result).toEqual({ isDelegated: false });
  });

  it("getLatestBlockhashForTransaction returns blockhash", async () => {
    (global.fetch as jest.Mock).mockResolvedValueOnce({
      json: async () => ({
        result: { blockhash: "mock-blockhash", lastValidBlockHeight: 100 },
      }),
    });

    const result = await (connection as any).getLatestBlockhashForTransaction(
      tx,
    );
    expect(result).toEqual({
      blockhash: "mock-blockhash",
      lastValidBlockHeight: 100,
    });
  });

  it("prepareTransaction sets recentBlockhash", async () => {
    // Spy on the prototype method
    jest
      .spyOn(Connection.prototype as any, "getLatestBlockhashForTransaction")
      .mockResolvedValue({ blockhash: "hb", lastValidBlockHeight: 100 });

    const result = await connection.prepareTransaction(tx);
    expect(result.recentBlockhash).toBe("hb");
  });

  it("sendTransaction signs and sends transaction", async () => {
    jest
      .spyOn(Connection.prototype as any, "getLatestBlockhashForTransaction")
      .mockResolvedValue({ blockhash: "hb", lastValidBlockHeight: 100 });

    jest
      .spyOn(Connection.prototype as any, "sendRawTransaction")
      .mockResolvedValue("sig123");

    const signers = [new Keypair()];

    const sendTx = connection.sendTransaction.bind(connection);
    const signature = await sendTx(tx, signers);

    const signFn = (tx as any).sign.bind(tx);
    const serializeFn = (tx as any).serialize.bind(tx);

    expect(signFn(...signers)).toBeUndefined();
    expect(serializeFn()).toBeInstanceOf(Buffer);
    expect(signature).toBe("sig123");
  });

  it("sendAndConfirmTransaction calls sendTransaction and returns signature", async () => {
    jest
      .spyOn(Connection.prototype as any, "sendTransaction")
      .mockResolvedValue("sig123");
    jest
      .spyOn(Connection.prototype as any, "confirmTransaction")
      .mockResolvedValue({ value: { err: null } });

    const signature = await connection.sendAndConfirmTransaction(tx, [
      new Keypair(),
    ]);

    expect(signature).toBe("sig123");
  });

  it("sendAndConfirmTransaction throws SendTransactionError if status has err", async () => {
    jest
      .spyOn(Connection.prototype as any, "sendTransaction")
      .mockResolvedValue("sig123");
    jest
      .spyOn(Connection.prototype as any, "confirmTransaction")
      .mockResolvedValue({ value: { err: { some: "error" } } });

    await expect(
      connection.sendAndConfirmTransaction(tx, [new Keypair()]),
    ).rejects.toThrow(SendTransactionError);
  });
});
