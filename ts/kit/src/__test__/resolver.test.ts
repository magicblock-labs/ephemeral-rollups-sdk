import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import type {
  AccountInfoBase,
  AccountInfoWithBase64EncodedData,
  Address,
} from "@solana/kit";
import { Resolver } from "../resolver";
import { DELEGATION_PROGRAM_ID } from "../constants";

type AccountInfo = (AccountInfoBase & AccountInfoWithBase64EncodedData) | null;

interface AccountNotification {
  context: { slot: bigint };
  value: AccountInfo;
}

const kitMocks = vi.hoisted(() => {
  const delegationRecordAddress =
    "DelegationRecord111111111111111111111111111" as Address;
  const validatorAddress =
    "Validator1111111111111111111111111111111" as Address;
  const decodeDelegationRecord = vi.fn(() => ({ validator: validatorAddress }));

  return {
    address: vi.fn((value: string) => value as Address),
    createSolanaRpc: vi.fn(),
    createSolanaRpcSubscriptions: vi.fn(),
    decodeDelegationRecord,
    delegationRecordAddress,
    getAddressCodec: vi.fn(() => ({})),
    getAddressEncoder: vi.fn(() => ({
      encode: vi.fn(() => new Uint8Array([1])),
    })),
    getProgramDerivedAddress: vi.fn(async () => [delegationRecordAddress]),
    getStructCodec: vi.fn(() => ({
      decode: vi.fn(() => decodeDelegationRecord()),
    })),
    getU8Codec: vi.fn(() => ({})),
    lamports: vi.fn((value: bigint) => value),
    validatorAddress,
  };
});

vi.mock("@solana/kit", () => ({
  AccountRole: {
    READONLY: 0,
    READONLY_SIGNER: 1,
    WRITABLE: 2,
    WRITABLE_SIGNER: 3,
  },
  address: kitMocks.address,
  createSolanaRpc: kitMocks.createSolanaRpc,
  createSolanaRpcSubscriptions: kitMocks.createSolanaRpcSubscriptions,
  getAddressCodec: kitMocks.getAddressCodec,
  getAddressEncoder: kitMocks.getAddressEncoder,
  getProgramDerivedAddress: kitMocks.getProgramDerivedAddress,
  getStructCodec: kitMocks.getStructCodec,
  getU8Codec: kitMocks.getU8Codec,
  lamports: kitMocks.lamports,
}));

function createNotificationStream() {
  const values: AccountNotification[] = [];
  const waiters: Array<{
    abortSignal?: AbortSignal;
    onAbort?: () => void;
    resolve: (result: IteratorResult<AccountNotification>) => void;
  }> = [];
  let closed = false;

  const resolveWaiter = (
    waiter: (typeof waiters)[number],
    result: IteratorResult<AccountNotification>,
  ) => {
    if (waiter.onAbort !== undefined) {
      waiter.abortSignal?.removeEventListener("abort", waiter.onAbort);
    }
    waiter.resolve(result);
  };

  const stream = (
    abortSignal?: AbortSignal,
  ): AsyncIterable<AccountNotification> => ({
    [Symbol.asyncIterator]() {
      return {
        async next() {
          if (closed || abortSignal?.aborted === true) {
            return { done: true, value: undefined };
          }

          return new Promise<IteratorResult<AccountNotification>>((resolve) => {
            const value = values.shift();
            if (value !== undefined) {
              resolve({ done: false, value });
              return;
            }

            const waiter: (typeof waiters)[number] = {
              abortSignal,
              resolve,
            };
            waiter.onAbort = () => {
              const index = waiters.indexOf(waiter);
              if (index !== -1) {
                waiters.splice(index, 1);
              }
              resolveWaiter(waiter, { done: true, value: undefined });
            };
            abortSignal?.addEventListener("abort", waiter.onAbort, {
              once: true,
            });
            waiters.push(waiter);
          });
        },
      };
    },
  });

  return {
    push(value: AccountNotification) {
      const waiter = waiters.shift();
      if (waiter !== undefined) {
        resolveWaiter(waiter, { done: false, value });
        return;
      }
      values.push(value);
    },
    close() {
      closed = true;
      for (const waiter of waiters.splice(0)) {
        resolveWaiter(waiter, { done: true, value: undefined });
      }
    },
    stream,
  };
}

function createAccountNotification(
  slot: bigint,
  value: AccountInfo,
): AccountNotification {
  return {
    context: { slot },
    value,
  };
}

function createDelegatedAccount(): AccountInfoBase &
  AccountInfoWithBase64EncodedData {
  return {
    data: ["AAAA"],
    executable: false,
    lamports: 1n,
    owner: DELEGATION_PROGRAM_ID,
    space: 0n,
  } as unknown as AccountInfoBase & AccountInfoWithBase64EncodedData;
}

describe("Resolver", () => {
  const accountAddress = "Account11111111111111111111111111111111" as Address;
  const chainUrl = "https://base.example";
  const routeUrl = "https://er.example";
  const routeRpc = { endpoint: routeUrl };

  let notificationStream: ReturnType<typeof createNotificationStream>;
  let getAccountInfo: ReturnType<typeof vi.fn>;
  let getAccountInfoSend: ReturnType<typeof vi.fn>;
  let accountNotifications: ReturnType<typeof vi.fn>;
  let subscribe: ReturnType<typeof vi.fn>;
  let chainRpc: { getAccountInfo: ReturnType<typeof vi.fn> };
  let resolver: Resolver | undefined;

  beforeEach(() => {
    vi.clearAllMocks();

    notificationStream = createNotificationStream();
    getAccountInfoSend = vi.fn(async () => createAccountNotification(1n, null));
    getAccountInfo = vi.fn(() => ({ send: getAccountInfoSend }));
    accountNotifications = vi.fn(() => ({ subscribe }));
    subscribe = vi.fn(async ({ abortSignal }: { abortSignal: AbortSignal }) =>
      notificationStream.stream(abortSignal),
    );
    chainRpc = { getAccountInfo };

    kitMocks.decodeDelegationRecord.mockReturnValue({
      validator: kitMocks.validatorAddress,
    });
    kitMocks.createSolanaRpc.mockImplementation((url: string) => {
      if (url === routeUrl) {
        return routeRpc;
      }
      return chainRpc;
    });
    kitMocks.createSolanaRpcSubscriptions.mockReturnValue({
      accountNotifications,
    });
  });

  afterEach(() => {
    resolver?.terminate();
    notificationStream.close();
    resolver = undefined;
  });

  it("returns from initial fetch without waiting for a websocket notification", async () => {
    const currentResolver = new Resolver(
      { chain: chainUrl, websocket: "wss://base.example" },
      new Map(),
    );
    resolver = currentResolver;

    const result = await currentResolver.trackAccount(accountAddress);

    expect(result).toEqual({ status: 1 });
    expect(accountNotifications).toHaveBeenCalledWith(
      kitMocks.delegationRecordAddress,
      {
        commitment: "confirmed",
        encoding: "base64",
      },
    );
    expect(subscribe).toHaveBeenCalledWith({
      abortSignal: expect.any(AbortSignal),
    });
    expect(getAccountInfo).toHaveBeenCalledWith(
      kitMocks.delegationRecordAddress,
      {
        commitment: "confirmed",
        encoding: "base64",
      },
    );
  });

  it("does not hang when no websocket notification arrives immediately", async () => {
    let resolveInitialFetch: ((value: AccountNotification) => void) | undefined;
    getAccountInfoSend.mockReturnValue(
      new Promise((resolve) => {
        resolveInitialFetch = resolve;
      }),
    );
    const currentResolver = new Resolver(
      { chain: chainUrl, websocket: "wss://base.example" },
      new Map(),
    );
    resolver = currentResolver;

    const trackAccount = currentResolver.trackAccount(accountAddress);
    await vi.waitFor(() => {
      expect(getAccountInfo).toHaveBeenCalled();
    });

    resolveInitialFetch?.(createAccountNotification(1n, null));
    await expect(trackAccount).resolves.toEqual({ status: 1 });
  });

  it("deduplicates concurrent tracking for the same account", async () => {
    let resolveInitialFetch: ((value: AccountNotification) => void) | undefined;
    getAccountInfoSend.mockReturnValue(
      new Promise((resolve) => {
        resolveInitialFetch = resolve;
      }),
    );
    const currentResolver = new Resolver(
      { chain: chainUrl, websocket: "wss://base.example" },
      new Map(),
    );
    resolver = currentResolver;

    const firstTrack = currentResolver.trackAccount(accountAddress);
    const secondTrack = currentResolver.trackAccount(accountAddress);
    await vi.waitFor(() => {
      expect(getAccountInfo).toHaveBeenCalled();
    });

    expect(accountNotifications).toHaveBeenCalledTimes(1);
    expect(subscribe).toHaveBeenCalledTimes(1);

    resolveInitialFetch?.(createAccountNotification(1n, null));
    await expect(Promise.all([firstTrack, secondTrack])).resolves.toEqual([
      { status: 1 },
      { status: 1 },
    ]);
  });

  it("aborts the subscription when the initial fetch fails", async () => {
    const fetchError = new Error("initial fetch failed");
    getAccountInfoSend.mockRejectedValue(fetchError);
    const currentResolver = new Resolver(
      { chain: chainUrl, websocket: "wss://base.example" },
      new Map(),
    );
    resolver = currentResolver;

    await expect(currentResolver.trackAccount(accountAddress)).rejects.toThrow(
      fetchError,
    );

    const abortSignal = subscribe.mock.calls[0][0].abortSignal as AbortSignal;
    expect(abortSignal.aborted).toBe(true);
  });

  it("continues tracking account updates after returning initial state", async () => {
    const currentResolver = new Resolver(
      { chain: chainUrl, websocket: "wss://base.example" },
      new Map([[kitMocks.validatorAddress, routeUrl]]),
    );
    resolver = currentResolver;

    await expect(currentResolver.trackAccount(accountAddress)).resolves.toEqual(
      {
        status: 1,
      },
    );

    notificationStream.push(
      createAccountNotification(2n, createDelegatedAccount()),
    );

    await vi.waitFor(async () => {
      await expect(currentResolver.resolve(accountAddress)).resolves.toBe(
        routeRpc,
      );
    });
  });

  it("does not overwrite a same-slot websocket notification with the initial fetch", async () => {
    let resolveInitialFetch: ((value: AccountNotification) => void) | undefined;
    getAccountInfoSend.mockReturnValue(
      new Promise((resolve) => {
        resolveInitialFetch = resolve;
      }),
    );
    const currentResolver = new Resolver(
      { chain: chainUrl, websocket: "wss://base.example" },
      new Map([[kitMocks.validatorAddress, routeUrl]]),
    );
    resolver = currentResolver;

    const trackAccount = currentResolver.trackAccount(accountAddress);
    await vi.waitFor(() => {
      expect(getAccountInfo).toHaveBeenCalled();
    });

    notificationStream.push(
      createAccountNotification(2n, createDelegatedAccount()),
    );
    await vi.waitFor(() => {
      expect(kitMocks.decodeDelegationRecord).toHaveBeenCalled();
    });

    resolveInitialFetch?.(createAccountNotification(2n, null));

    await expect(trackAccount).resolves.toEqual({
      status: 0,
      validator: kitMocks.validatorAddress,
    });
    await expect(currentResolver.resolve(accountAddress)).resolves.toBe(
      routeRpc,
    );
  });

  it("terminates active account subscriptions", async () => {
    const currentResolver = new Resolver(
      { chain: chainUrl, websocket: "wss://base.example" },
      new Map(),
    );
    resolver = currentResolver;

    await currentResolver.trackAccount(accountAddress);
    const abortSignal = subscribe.mock.calls[0][0].abortSignal as AbortSignal;

    expect(abortSignal.aborted).toBe(false);

    currentResolver.terminate();

    expect(abortSignal.aborted).toBe(true);
  });
});
