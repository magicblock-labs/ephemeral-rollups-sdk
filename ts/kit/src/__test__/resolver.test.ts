import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import type {
  AccountInfoBase,
  AccountInfoWithBase64EncodedData,
  Address,
} from "@solana/kit";
import { Resolver } from "../resolver";
import { DELEGATION_PROGRAM_ID } from "../constants";

interface AccountNotification {
  value: (AccountInfoBase & AccountInfoWithBase64EncodedData) | null;
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
  const waiters: Array<(result: IteratorResult<AccountNotification>) => void> =
    [];
  let closed = false;

  const stream: AsyncIterable<AccountNotification> = {
    [Symbol.asyncIterator]() {
      return {
        async next() {
          if (closed) {
            return { done: true, value: undefined };
          }

          const value = values.shift();
          if (value !== undefined) {
            return { done: false, value };
          }

          return new Promise<IteratorResult<AccountNotification>>((resolve) => {
            waiters.push(resolve);
          });
        },
      };
    },
  };

  return {
    push(value: AccountNotification) {
      const waiter = waiters.shift();
      if (waiter !== undefined) {
        waiter({ done: false, value });
        return;
      }
      values.push(value);
    },
    close() {
      closed = true;
      for (const waiter of waiters.splice(0)) {
        waiter({ done: true, value: undefined });
      }
    },
    stream,
  };
}

async function timeout(ms: number): Promise<"timeout"> {
  return new Promise((resolve) => {
    setTimeout(() => {
      resolve("timeout");
    }, ms);
  });
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

  beforeEach(() => {
    vi.clearAllMocks();

    notificationStream = createNotificationStream();
    getAccountInfoSend = vi.fn(async () => ({ value: null }));
    getAccountInfo = vi.fn(() => ({ send: getAccountInfoSend }));
    accountNotifications = vi.fn(() => ({ subscribe }));
    subscribe = vi.fn(async () => notificationStream.stream);
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
    notificationStream.close();
  });

  it("returns from initial fetch without waiting for a websocket notification", async () => {
    const resolver = new Resolver(
      { chain: chainUrl, websocket: "wss://base.example" },
      new Map(),
    );

    const result = await Promise.race([
      resolver.trackAccount(accountAddress),
      timeout(50),
    ]);

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
    let resolveInitialFetch: ((value: { value: null }) => void) | undefined;
    getAccountInfoSend.mockReturnValue(
      new Promise((resolve) => {
        resolveInitialFetch = resolve;
      }),
    );
    const resolver = new Resolver(
      { chain: chainUrl, websocket: "wss://base.example" },
      new Map(),
    );

    const trackAccount = resolver.trackAccount(accountAddress);
    await vi.waitFor(() => {
      expect(getAccountInfo).toHaveBeenCalled();
    });

    resolveInitialFetch?.({ value: null });
    await expect(trackAccount).resolves.toEqual({ status: 1 });
  });

  it("continues tracking account updates after returning initial state", async () => {
    const resolver = new Resolver(
      { chain: chainUrl, websocket: "wss://base.example" },
      new Map([[kitMocks.validatorAddress, routeUrl]]),
    );

    await expect(resolver.trackAccount(accountAddress)).resolves.toEqual({
      status: 1,
    });

    notificationStream.push({
      value: {
        data: ["AAAA"],
        executable: false,
        lamports: 1n,
        owner: DELEGATION_PROGRAM_ID,
        space: 0n,
      } as unknown as AccountInfoBase & AccountInfoWithBase64EncodedData,
    });

    await vi.waitFor(async () => {
      await expect(resolver.resolve(accountAddress)).resolves.toBe(routeRpc);
    });
  });
});
