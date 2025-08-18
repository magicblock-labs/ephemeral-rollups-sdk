import {
  prepareMagicTransaction,
  sendMagicTransaction,
  getWritableAccounts,
  getClosestValidator,
  getDelegationStatus,
} from "../magic-router.js";
import { Connection, Transaction, Keypair, PublicKey } from "@solana/web3.js";

// Mock PublicKey class
const mockPublicKey = (address: string) => ({
  toBase58: () => address,
  toString: () => address,
});

jest.mock("@solana/web3.js", () => {
  const actual = jest.requireActual("@solana/web3.js");
  return {
    ...actual,
    Connection: jest.fn().mockImplementation(() => ({
      rpcEndpoint: "http://localhost",
      sendRawTransaction: jest.fn().mockResolvedValue("mock-signature"),
    })),
    Transaction: jest.fn().mockImplementation(() => ({
      feePayer: mockPublicKey("mock-fee-payer"),
      signature: [],
      instructions: [
        {
          keys: [
            { pubkey: mockPublicKey("key1"), isSigner: true, isWritable: true },
            {
              pubkey: mockPublicKey("key2"),
              isSigner: false,
              isWritable: false,
            },
          ],
        },
      ],
      serialize: jest.fn(() => Buffer.from("mock")),
      sign: jest.fn(),
    })),
    Keypair: jest.fn().mockImplementation(() => ({
      publicKey: mockPublicKey("mock-public-key"),
      sign: jest.fn(),
    })),
    PublicKey: jest
      .fn()
      .mockImplementation((address: string) => mockPublicKey(address)),
  };
});

global.fetch = jest.fn(async () =>
  Promise.resolve({
    json: async () =>
      Promise.resolve({ result: { blockhash: "mock-blockhash" } }),
  }),
) as any;

describe("prepareRouterTransaction", () => {
  it("sets recentBlockhash and returns the transaction", async () => {
    const connection = new Connection("http://localhost");
    const transaction = new Transaction();
    const result = await prepareMagicTransaction(connection, transaction);
    expect(result.recentBlockhash).toBe("mock-blockhash");
    expect(global.fetch).toHaveBeenCalledWith("http://localhost", {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({
        jsonrpc: "2.0",
        id: 1,
        method: "getBlockhashForAccounts",
        params: [["mock-fee-payer", "key1"]],
      }),
    });
  });
});

describe("sendRouterTransaction", () => {
  it("sets recentBlockhash, feePayer, signs, and sends the transaction", async () => {
    const connection = new Connection("http://localhost");
    const transaction = new Transaction();
    const signers = [new Keypair()];
    const signature = await sendMagicTransaction(
      connection,
      transaction,
      signers,
    );

    expect(transaction.recentBlockhash).toBe("mock-blockhash");
    expect(transaction.feePayer?.toBase58()).toBe("mock-fee-payer");
    expect((transaction as any).sign).toHaveBeenCalledWith(...signers);
    expect(signature).toBe("mock-signature");
    expect(global.fetch).toHaveBeenCalledWith("http://localhost", {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({
        jsonrpc: "2.0",
        id: 1,
        method: "getBlockhashForAccounts",
        params: [["mock-fee-payer", "key1"]],
      }),
    });
  });
});

describe("getWritableAccounts", () => {
  it("returns writable accounts from transaction", () => {
    const transaction = {
      feePayer: mockPublicKey("fee-payer"),
      instructions: [
        {
          keys: [
            { pubkey: mockPublicKey("key1"), isWritable: true },
            { pubkey: mockPublicKey("key2"), isWritable: false },
            { pubkey: mockPublicKey("key3"), isWritable: true },
          ],
        },
      ],
    } as unknown as Transaction;

    const result = getWritableAccounts(transaction);
    expect(result).toEqual(["fee-payer", "key1", "key3"]);
  });

  it("handles transaction without feePayer", () => {
    const transaction = {
      feePayer: null,
      instructions: [
        {
          keys: [
            { pubkey: mockPublicKey("key1"), isWritable: true },
            { pubkey: mockPublicKey("key2"), isWritable: false },
          ],
        },
      ],
    } as unknown as Transaction;

    const result = getWritableAccounts(transaction);
    expect(result).toEqual(["key1"]);
  });

  it("handles transaction without instructions", () => {
    const transaction = {
      feePayer: mockPublicKey("fee-payer"),
      instructions: [],
    } as unknown as Transaction;

    const result = getWritableAccounts(transaction);
    expect(result).toEqual(["fee-payer"]);
  });

  it("deduplicates writable accounts", () => {
    const transaction = {
      feePayer: mockPublicKey("fee-payer"),
      instructions: [
        {
          keys: [
            { pubkey: mockPublicKey("key1"), isWritable: true },
            { pubkey: mockPublicKey("key1"), isWritable: true }, // Duplicate
            { pubkey: mockPublicKey("key2"), isWritable: false },
          ],
        },
      ],
    } as unknown as Transaction;

    const result = getWritableAccounts(transaction);
    expect(result).toEqual(["fee-payer", "key1"]);
  });
});

describe("getClosestValidator", () => {
  beforeEach(() => {
    jest.clearAllMocks();
  });

  it("fetches and returns the closest validator public key", async () => {
    const mockIdentityData = {
      result: {
        identity: "mock-validator-identity",
      },
    };

    (global.fetch as jest.Mock).mockResolvedValueOnce({
      json: async () => Promise.resolve(mockIdentityData),
    });

    const connection = new Connection("http://localhost");
    const result = await getClosestValidator(connection);

    expect(global.fetch).toHaveBeenCalledWith("http://localhost", {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({
        jsonrpc: "2.0",
        id: 1,
        method: "getIdentity",
        params: [],
      }),
    });

    expect(result.toBase58()).toBe("mock-validator-identity");
  });

  it("handles fetch errors gracefully", async () => {
    (global.fetch as jest.Mock).mockRejectedValueOnce(
      new Error("Network error"),
    );

    const connection = new Connection("http://localhost");

    await expect(getClosestValidator(connection)).rejects.toThrow(
      "Network error",
    );
  });

  it("handles invalid response format", async () => {
    (global.fetch as jest.Mock).mockResolvedValueOnce({
      json: async () => Promise.resolve({ error: "Invalid response" }),
    });

    const connection = new Connection("http://localhost");

    await expect(getClosestValidator(connection)).rejects.toThrow();
  });
});

describe("getDelegationStatus", () => {
  beforeEach(() => {
    jest.clearAllMocks();
  });

  it("returns delegation status for a string account", async () => {
    (global.fetch as jest.Mock).mockResolvedValueOnce({
      json: async () => Promise.resolve({ result: { isDelegated: false } }),
    });

    const connection = new Connection("http://localhost");
    const account = "7xKXtg2CW87d97TXJSDpbD5jBkheTqA83TZRuJosgAsU";

    const result = await getDelegationStatus(connection, account);

    expect(global.fetch).toHaveBeenCalledWith(
      "http://localhost/getDelegationStatus",
      {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({
          jsonrpc: "2.0",
          id: 1,
          method: "getDelegationStatus",
          params: [account],
        }),
      },
    );

    expect(result).toEqual({ isDelegated: false });
  });

  it("returns delegation status for a PublicKey account", async () => {
    (global.fetch as jest.Mock).mockResolvedValueOnce({
      json: async () => Promise.resolve({ result: { isDelegated: true } }),
    });

    const connection = new Connection("http://localhost");
    const accountKey = new PublicKey("mock-public-key");

    const result = await getDelegationStatus(connection, accountKey);

    expect(global.fetch).toHaveBeenCalledWith(
      "http://localhost/getDelegationStatus",
      {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({
          jsonrpc: "2.0",
          id: 1,
          method: "getDelegationStatus",
          params: [accountKey.toBase58()],
        }),
      },
    );

    expect(result).toEqual({ isDelegated: true });
  });
});
