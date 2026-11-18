import { afterEach, describe, expect, it, vi } from "vitest";
import type { Connection, ParsedTransactionWithMeta } from "@solana/web3.js";
import { MAGIC_PROGRAM_ID } from "../constants";
import { GetCommitmentSignature } from "../utils";

const schedulingSignature = "scheduling-signature";
const scheduledCommitSignature = "scheduled-commit-signature";
const commitmentSignature = "commitment-signature";
const outboxAddress = "11111111111111111111111111111111";

function transactionWithLogs(logMessage: string): ParsedTransactionWithMeta {
  const programId = MAGIC_PROGRAM_ID.toBase58();
  return {
    meta: {
      err: null,
      logMessages: [
        `Program ${programId} invoke [1]`,
        `Program log: ${logMessage}`,
        `Program ${programId} success`,
      ],
    },
  } as unknown as ParsedTransactionWithMeta;
}

function createConnection(
  getParsedTransaction: ReturnType<typeof vi.fn>,
  getSignaturesForAddress = vi.fn(),
) {
  return {
    getParsedTransaction,
    getSignaturesForAddress,
    getLatestBlockhash: vi.fn(async () => ({
      blockhash: "blockhash",
      lastValidBlockHeight: 100,
    })),
    confirmTransaction: vi.fn(async () => ({
      context: { slot: 1 },
      value: { err: null },
    })),
  } as unknown as Connection;
}

afterEach(() => {
  vi.useRealTimers();
});

describe("GetCommitmentSignature transaction lookup", () => {
  it("retries missing transaction metadata until it is available", async () => {
    vi.useFakeTimers();
    const getParsedTransaction = vi
      .fn()
      .mockResolvedValueOnce(null)
      .mockResolvedValueOnce({ meta: null })
      .mockResolvedValueOnce(
        transactionWithLogs(
          `ScheduledCommitSent signature: ${scheduledCommitSignature}`,
        ),
      )
      .mockResolvedValueOnce(
        transactionWithLogs(
          `ScheduledCommitSent signature[0]: ${commitmentSignature}`,
        ),
      );
    const connection = createConnection(getParsedTransaction);

    const result = GetCommitmentSignature(schedulingSignature, connection);
    await vi.advanceTimersByTimeAsync(2_000);

    await expect(result).resolves.toBe(commitmentSignature);
    expect(getParsedTransaction).toHaveBeenCalledTimes(4);
    expect(getParsedTransaction).toHaveBeenLastCalledWith(
      scheduledCommitSignature,
      {
        commitment: "confirmed",
        maxSupportedTransactionVersion: 1,
      },
    );
  });

  it("fails after the scheduling transaction remains unavailable", async () => {
    vi.useFakeTimers();
    const getParsedTransaction = vi.fn().mockResolvedValue(null);
    const connection = createConnection(getParsedTransaction);
    const result = GetCommitmentSignature(schedulingSignature, connection);
    const rejection = expect(result).rejects.toThrow(
      "Transaction not found or meta is null after waiting 30000ms",
    );

    await vi.advanceTimersByTimeAsync(30_000);

    await rejection;
    expect(getParsedTransaction).toHaveBeenCalledTimes(30);
  });

  it("times out when the settlement signature lookup remains pending", async () => {
    vi.useFakeTimers();
    const getParsedTransaction = vi
      .fn()
      .mockResolvedValue(
        transactionWithLogs(`Outbox for scheduled intent: ${outboxAddress}`),
      );
    const getSignaturesForAddress = vi.fn(
      async () => new Promise<never>(() => {}),
    );
    const connection = createConnection(
      getParsedTransaction,
      getSignaturesForAddress,
    );

    const result = GetCommitmentSignature(schedulingSignature, connection);
    const rejection = expect(result).rejects.toThrow(
      `Timed out waiting for settlement outbox ${outboxAddress}`,
    );
    await vi.advanceTimersByTimeAsync(120_000);

    await rejection;
    expect(getSignaturesForAddress).toHaveBeenCalledTimes(1);
  });

  it("times out when a settlement transaction lookup remains pending", async () => {
    vi.useFakeTimers();
    const getParsedTransaction = vi
      .fn()
      .mockResolvedValueOnce(
        transactionWithLogs(`Outbox for scheduled intent: ${outboxAddress}`),
      )
      .mockImplementationOnce(async () => new Promise<never>(() => {}));
    const getSignaturesForAddress = vi
      .fn()
      .mockResolvedValue([{ signature: "receipt-transaction" }]);
    const connection = createConnection(
      getParsedTransaction,
      getSignaturesForAddress,
    );

    const result = GetCommitmentSignature(schedulingSignature, connection);
    const rejection = expect(result).rejects.toThrow(
      `Timed out waiting for settlement outbox ${outboxAddress}`,
    );
    await vi.advanceTimersByTimeAsync(120_000);

    await rejection;
    expect(getParsedTransaction).toHaveBeenCalledTimes(2);
  });
});
