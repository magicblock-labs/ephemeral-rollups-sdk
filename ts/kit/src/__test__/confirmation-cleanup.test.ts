import { afterEach, beforeEach, expect, it, vi } from "vitest";
import { signature } from "@solana/kit";
import { getTimeoutPromise } from "@solana/transaction-confirmation";
import {
  Connection,
  waitForRecentTransactionConfirmationUntilTimeout,
} from "../index";

const transport = vi.hoisted(() => {
  const signals: AbortSignal[] = [];
  return { signals };
});

// Keep the real RPC client, confirmation strategies and timeout implementation.
// Only the subscription transport is replaced by an abort-aware pending stream.
vi.mock("@solana/kit", async (importOriginal) => {
  const actual = await importOriginal<typeof import("@solana/kit")>();
  return {
    ...actual,
    createSolanaRpcSubscriptions: () => ({
      signatureNotifications: () => ({
        subscribe: async ({ abortSignal }: { abortSignal: AbortSignal }) => {
          abortSignal.throwIfAborted();
          transport.signals.push(abortSignal);
          return {
            [Symbol.asyncIterator]() {
              return this;
            },
            next: async () =>
              new Promise<IteratorResult<never>>((_resolve, reject) => {
                const abort = () => {
                  reject(new DOMException("Aborted", "AbortError"));
                };
                if (abortSignal.aborted) abort();
                else
                  abortSignal.addEventListener("abort", abort, { once: true });
              }),
          };
        },
      }),
    }),
  };
});

const txSignature = signature("1".repeat(64));

async function connectionFor(status: "confirmed" | "pending" | "failed") {
  vi.stubGlobal("fetch", async (_url: string, init: RequestInit) => {
    const request: unknown = JSON.parse(String(init.body));
    if (
      typeof request !== "object" ||
      request === null ||
      !("method" in request) ||
      !("id" in request)
    ) {
      throw new Error("Unexpected JSON-RPC request");
    }
    if (request.method === "getBlockhashForAccounts") {
      return Response.json({
        jsonrpc: "2.0",
        id: request.id,
        error: { code: -32601, message: "Method not found" },
      });
    }
    expect(request.method).toBe("getSignatureStatuses");
    return Response.json({
      jsonrpc: "2.0",
      id: request.id,
      result: {
        context: { slot: 100 },
        value: [
          status === "pending"
            ? null
            : {
                slot: 99,
                confirmations: status === "failed" ? 0 : 1,
                err:
                  status === "failed"
                    ? { InstructionError: [0, "InvalidArgument"] }
                    : null,
                confirmationStatus:
                  status === "failed" ? "processed" : "confirmed",
              },
        ],
      },
    });
  });
  return Connection.create("http://127.0.0.1:8899/");
}

beforeEach(() => {
  vi.useFakeTimers();
  transport.signals.length = 0;
});

afterEach(() => {
  vi.useRealTimers();
  vi.unstubAllGlobals();
});

it("direct race control cancels its losing timeout", async () => {
  await waitForRecentTransactionConfirmationUntilTimeout({
    signature: txSignature,
    commitment: "confirmed",
    getTimeoutPromise,
    getRecentSignatureConfirmationPromise: async () => {},
  });
  expect(vi.getTimerCount()).toBe(0);
});

it.each(["confirmed", "failed"] as const)(
  "cleans up after a %s status without aborting the caller",
  async (status) => {
    const connection = await connectionFor(status);
    const caller = new AbortController();
    try {
      const pending = connection.confirmTransaction(txSignature, {
        abortSignal: caller.signal,
      });
      if (status === "confirmed")
        await expect(pending).resolves.toBeUndefined();
      else await expect(pending).rejects.toThrow();
      expect(transport.signals).toHaveLength(1);
      expect(transport.signals[0].aborted).toBe(true);
      expect(caller.signal.aborted).toBe(false);
      expect(vi.getTimerCount()).toBe(0);
    } finally {
      caller.abort();
    }
  },
);

it("cancels the signature subscription when the timeout wins", async () => {
  const connection = await connectionFor("pending");
  const caller = new AbortController();
  try {
    const result = connection
      .confirmTransaction(txSignature, { abortSignal: caller.signal })
      .catch((error: unknown) => error);
    await vi.advanceTimersByTimeAsync(60_000);
    expect(await result).toMatchObject({ name: "TimeoutError" });
    expect(transport.signals).toHaveLength(1);
    expect(transport.signals[0].aborted).toBe(true);
    expect(caller.signal.aborted).toBe(false);
    expect(vi.getTimerCount()).toBe(0);
  } finally {
    caller.abort();
  }
});

it("honors caller cancellation while confirmation is pending", async () => {
  const connection = await connectionFor("pending");
  const caller = new AbortController();
  const result = connection
    .confirmTransaction(txSignature, { abortSignal: caller.signal })
    .catch((error: unknown) => error);
  await vi.advanceTimersByTimeAsync(0);
  expect(transport.signals).toHaveLength(1);
  caller.abort();
  expect(await result).toMatchObject({ name: "AbortError" });
  expect(transport.signals[0].aborted).toBe(true);
  expect(vi.getTimerCount()).toBe(0);
});

it("rejects a pre-aborted call without starting strategies", async () => {
  const connection = await connectionFor("pending");
  const caller = new AbortController();
  caller.abort();
  let settled = false;
  const result = connection
    .confirmTransaction(txSignature, { abortSignal: caller.signal })
    .catch((error: unknown) => error)
    .finally(() => {
      settled = true;
    });
  try {
    await vi.advanceTimersByTimeAsync(0);
    expect(settled).toBe(true);
    expect(await result).toMatchObject({ name: "AbortError" });
    expect(transport.signals).toHaveLength(0);
    expect(vi.getTimerCount()).toBe(0);
  } finally {
    await vi.runAllTimersAsync();
  }
});
