import { describe, it, expect, vi, beforeEach } from 'vitest';
import { Connection } from '../connection'; // adjust path
import * as utils from '../utils';
import * as solanaKit from "@solana/kit";
import * as confirmation from "../confirmation";
import { createRecentSignatureConfirmationPromiseFactory, getTimeoutPromise } from "@solana/transaction-confirmation";

// ----------------------
// Nominal type mocks
// ----------------------
const mockSignature = "mock-signature" as unknown as solanaKit.Signature;
const mockTxBytes = new Uint8Array() as unknown as solanaKit.TransactionMessageBytes;
const mockSerializedTx = "serialized-tx" as unknown as solanaKit.Base64EncodedWireTransaction;
const mockLifetimeConstraint = {
  blockhash: "bh" as unknown as solanaKit.Blockhash,
  lastValidBlockHeight: 999n
} as unknown as solanaKit.TransactionBlockhashLifetime;

// ----------------------
// RPC and Subscriptions mocks
// ----------------------
const mockRpc = {
  sendTransaction: vi.fn(() => ({ send: vi.fn(async () => mockSignature) })),
  getTransaction: vi.fn(() => ({ send: vi.fn(async () => ({ meta: { logMessages: ["log1"] } })) })),
  getLatestBlockhash: vi.fn(() => ({ send: vi.fn(async () => ({ value: { blockhash: "blockhash", lastValidBlockHeight: 100n } })) })),
  requestAirdrop: vi.fn(),
  getAccountInfo: vi.fn(),
  getBalance: vi.fn(),
  getBlock: vi.fn(),
  getBlockHeight: vi.fn(),
  getMultipleAccounts: vi.fn(),
  getSignaturesForAddress: vi.fn(),
  getTransactionCount: vi.fn(),
  getVersion: vi.fn(),
  getEpochInfo: vi.fn(),
  getLeaderSchedule: vi.fn(),
  getSlot: vi.fn(),
  getClusterNodes: vi.fn(),
  getInflationGovernor: vi.fn(),
  getSupply: vi.fn(),
  getVoteAccounts: vi.fn(),
  getFeeForMessage: vi.fn(),
  getFeeCalculatorForBlockhash: vi.fn(),
  getMinimumBalanceForRentExemption: vi.fn(),
  simulateTransaction: vi.fn(),
  getLargestAccounts: vi.fn(),
  getTokenAccountBalance: vi.fn(),
  getTokenAccountsByOwner: vi.fn(),
  getTokenSupply: vi.fn(),
  getProgramAccounts: vi.fn(),
  getRecentPerformanceSamples: vi.fn(),
  getConfirmedBlock: vi.fn(),
  getTransactionWithConfig: vi.fn(),
  getStakeActivation: vi.fn(),
  getIdentity: vi.fn(),
  getVoteAccount: vi.fn(),
} as any;

const mockRpcSubscriptions = {} as any;

const mockSigners = [{}] as any;

// ----------------------
// Vitest Mocks
// ----------------------
vi.mock('@solana/kit', () => ({
  createSolanaRpc: vi.fn(),
  createSolanaRpcSubscriptions: vi.fn(),
  partiallySignTransaction: vi.fn(),
  compileTransaction: vi.fn(),
  getBase64EncodedWireTransaction: vi.fn(),
  pipe: vi.fn((tx, fn) => fn(tx)),
  assertIsTransactionMessageWithBlockhashLifetime: vi.fn(),
  setTransactionMessageLifetimeUsingBlockhash: vi.fn((blockhash, tx) => tx)
}));

vi.mock('../utils', () => ({
  isRouter: vi.fn(),
  getWritableAccounts: vi.fn(),
  parseCommitsLogsMessage: vi.fn(),
  parseScheduleCommitsLogsMessage: vi.fn()
}));

vi.mock("../confirmation", () => ({
  waitForRecentTransactionConfirmationUntilTimeout: vi.fn(),
}));

vi.mock("@solana/transaction-confirmation", () => ({
  createRecentSignatureConfirmationPromiseFactory: vi.fn(),
  getTimeoutPromise: vi.fn(),
}));


// ----------------------
// Tests
// ----------------------
describe('Connection', () => {
  beforeEach(() => {
    vi.mocked(utils.isRouter).mockResolvedValue(false);
    vi.mocked(solanaKit.createSolanaRpc).mockReturnValue(mockRpc);
    vi.mocked(solanaKit.createSolanaRpcSubscriptions).mockReturnValue(mockRpcSubscriptions);
    vi.mocked(solanaKit.partiallySignTransaction).mockResolvedValue({
      messageBytes: mockTxBytes,
      signatures: {},
      lifetimeConstraint: mockLifetimeConstraint, // <- now type-safe
    });
    vi.mocked(solanaKit.compileTransaction).mockReturnValue({
      messageBytes: mockTxBytes,
      signatures: {},
    });
    vi.mocked(solanaKit.getBase64EncodedWireTransaction).mockReturnValue(mockSerializedTx);
    vi.mocked(createRecentSignatureConfirmationPromiseFactory).mockReturnValue(
      async () => undefined // must be a promise
    );
    vi.mocked(getTimeoutPromise).mockResolvedValue(undefined);
    vi.mocked(confirmation.waitForRecentTransactionConfirmationUntilTimeout).mockResolvedValue(undefined);
    vi.mocked(utils.getWritableAccounts).mockReturnValue(["account1"]);
    vi.mocked(utils.parseScheduleCommitsLogsMessage).mockReturnValue(mockSignature);
    vi.mocked(utils.parseCommitsLogsMessage).mockReturnValue(mockSignature);
  });

  it('should create a Connection instance', async () => {
    const connection = await Connection.create('http://localhost', 'ws://localhost');
    expect(connection).toBeInstanceOf(Connection);
    expect(connection.clusterUrlHttp).toBe('http://localhost');
    expect(connection.clusterUrlWs).toBe('ws://localhost');
    expect(connection.rpc).toBe(mockRpc);
    expect(connection.rpcSubscriptions).toBe(mockRpcSubscriptions);
    expect(connection.isMagicRouter).toBe(false);
  });

  it("should prepareTransactionWithLatestBlockhash", async () => {
    const connection = await Connection.create("http://localhost");
    const txMessage = { feePayer: "payer" } as any;
    const prepared = await connection.prepareTransactionWithLatestBlockhash(txMessage);
    expect(prepared).toEqual(txMessage);
  });

  it("should sendTransaction and return signature", async () => {
    const connection = await Connection.create("http://localhost");
    const txMessage = { feePayer: "payer", lifetimeConstraint: { blockhash: "bh" } } as any;
    const sig = await connection.sendTransaction(txMessage, mockSigners);
    expect(sig).toBe(mockSignature);
  });

  it("should confirmTransaction without errors", async () => {
    const connection = await Connection.create("http://localhost");
    await expect(connection.confirmTransaction(mockSignature)).resolves.not.toThrow();
    expect(confirmation.waitForRecentTransactionConfirmationUntilTimeout).toHaveBeenCalled();
  });

  it("should partiallySignTransaction", async () => {
    const connection = await Connection.create("http://localhost");
    const txMessage = { feePayer: "payer", lifetimeConstraint: { blockhash: "bh" } } as any;
    const partiallySigned = await connection.partiallySignTransaction(mockSigners, txMessage);
    expect(partiallySigned).toHaveProperty("messageBytes");
    expect(partiallySigned).toHaveProperty("signatures");
  });

  it("should sendAndConfirmTransaction", async () => {
    const connection = await Connection.create("http://localhost");
    const txMessage = { feePayer: "payer", lifetimeConstraint: { blockhash: "bh" } } as any;
    const sig = await connection.sendAndConfirmTransaction(txMessage, mockSigners, {});
    expect(sig).toBe(mockSignature);
  });

  it("should getLatestBlockhashForTransaction", async () => {
    const connection = await Connection.create("http://localhost");
    const txMessage = { feePayer: "payer" } as any;
    const blockhash = await connection.getLatestBlockhashForTransaction(txMessage);
    expect(blockhash.blockhash).toBe("blockhash");
    expect(blockhash.lastValidBlockHeight).toBe(100n);
  });

  it("should getCommitmentSignature", async () => {
    const connection = await Connection.create("http://localhost");
    const sig = await connection.getCommitmentSignature(mockSignature);
    expect(sig).toBe(mockSignature);
  });
});