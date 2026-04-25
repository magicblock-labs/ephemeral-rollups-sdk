import { describe, it, expect, vi, beforeEach, afterEach } from "vitest";
import { Connection } from "../connection";
import * as utils from "../utils";
import { postRouterRpc, RouterRpcError } from "../router-rpc";
import * as solanaKit from "@solana/kit";

// This suite exercises the real router RPC error handling in
// `getLatestBlockhashForTransaction` and `isRouter`, by stubbing the global
// `fetch` rather than mocking `../utils`.

vi.mock("@solana/kit", async () => {
  const actual = await vi.importActual<typeof solanaKit>("@solana/kit");
  return {
    ...actual,
    createSolanaRpc: vi.fn(),
    createSolanaRpcSubscriptions: vi.fn(),
  };
});

interface FetchResponse {
  ok: boolean;
  status: number;
  json: () => Promise<unknown>;
  text: () => Promise<string>;
}

function jsonResponse(body: unknown, status = 200): FetchResponse {
  return {
    ok: status >= 200 && status < 300,
    status,
    json: async () => body,
    text: async () => JSON.stringify(body),
  };
}

function textResponse(text: string, status: number): FetchResponse {
  return {
    ok: status >= 200 && status < 300,
    status,
    json: async () => {
      throw new Error("not json");
    },
    text: async () => text,
  };
}

const mockRpc = {
  getLatestBlockhash: vi.fn(),
} as unknown as solanaKit.Rpc<solanaKit.SolanaRpcApi>;

const mockRpcSubs =
  {} as unknown as solanaKit.RpcSubscriptions<solanaKit.SolanaRpcSubscriptionsApi>;

const fetchMock = vi.fn<(...args: unknown[]) => Promise<FetchResponse>>();

beforeEach(() => {
  vi.mocked(solanaKit.createSolanaRpc).mockReturnValue(mockRpc);
  vi.mocked(solanaKit.createSolanaRpcSubscriptions).mockReturnValue(
    mockRpcSubs,
  );
  fetchMock.mockReset();
  vi.stubGlobal("fetch", fetchMock);
});

afterEach(() => {
  vi.unstubAllGlobals();
});

/**
 * Builds a Connection instance that believes it is talking to a Magic Router,
 * without actually hitting the network during construction.
 */
async function buildRouterConnection(): Promise<Connection> {
  // Respond to the isRouter probe with a valid router result.
  fetchMock.mockResolvedValueOnce(
    jsonResponse({
      jsonrpc: "2.0",
      id: 1,
      result: { blockhash: "probe", lastValidBlockHeight: 1 },
    }),
  );
  const conn = await Connection.create("http://router.test");
  expect(conn.isMagicRouter).toBe(true);
  return conn;
}

const txMessage = {
  feePayer: "payer",
  instructions: [],
} as unknown as solanaKit.TransactionMessage &
  solanaKit.TransactionMessageWithFeePayer<string>;

describe("getLatestBlockhashForTransaction (Magic Router branch)", () => {
  it("surfaces structured JSON-RPC error as RouterRpcError with code, message, and data", async () => {
    const conn = await buildRouterConnection();

    const upstreamMessage =
      "account has been delegated to unknown ER node: mAGic...";
    fetchMock.mockResolvedValueOnce(
      jsonResponse({
        jsonrpc: "2.0",
        id: 1,
        error: {
          code: -32604,
          message: upstreamMessage,
          data: { node: "mAGic..." },
        },
      }),
    );

    const call = conn.getLatestBlockhashForTransaction(txMessage);
    await expect(call).rejects.toBeInstanceOf(RouterRpcError);
    await expect(call).rejects.toThrow(
      /-32604.*account has been delegated to unknown ER node/,
    );
    await call.catch((err: unknown) => {
      expect(err).toBeInstanceOf(RouterRpcError);
      const routerErr = err as RouterRpcError;
      expect(routerErr.code).toBe(-32604);
      expect(routerErr.method).toBe("getBlockhashForAccounts");
      expect(routerErr.data).toEqual({ node: "mAGic..." });
    });
  });

  it("does not swallow -32601 from getBlockhashForAccounts", async () => {
    // -32601 is classified as "not a router" only by the isRouter probe.
    // Once a Connection is latched as Magic Router, a -32601 from the
    // real RPC call must surface as a RouterRpcError, not silently
    // degrade.
    const conn = await buildRouterConnection();

    fetchMock.mockResolvedValueOnce(
      jsonResponse({
        jsonrpc: "2.0",
        id: 1,
        error: { code: -32601, message: "Method not found" },
      }),
    );

    const call = conn.getLatestBlockhashForTransaction(txMessage);
    await expect(call).rejects.toBeInstanceOf(RouterRpcError);
    await call.catch((err: unknown) => {
      expect((err as RouterRpcError).code).toBe(-32601);
    });
  });

  it("rejects when blockhash is the empty string", async () => {
    const conn = await buildRouterConnection();

    fetchMock.mockResolvedValueOnce(
      jsonResponse({
        jsonrpc: "2.0",
        id: 1,
        result: { blockhash: "", lastValidBlockHeight: 100 },
      }),
    );

    await expect(
      conn.getLatestBlockhashForTransaction(txMessage),
    ).rejects.toThrow(/Invalid getBlockhashForAccounts response/);
  });

  it("rejects when the 200 body is not JSON", async () => {
    const conn = await buildRouterConnection();

    const parseErr = new SyntaxError("non-json");
    fetchMock.mockResolvedValueOnce({
      ok: true,
      status: 200,
      json: async () => {
        throw parseErr;
      },
      text: async () => "<html>oops</html>",
    });

    const call = conn.getLatestBlockhashForTransaction(txMessage);
    await expect(call).rejects.toThrow(/returned non-JSON body/);
    await call.catch((err: unknown) => {
      expect((err as Error & { cause?: unknown }).cause).toBe(parseErr);
    });
  });

  it("rejects when the 200 body has neither result nor error", async () => {
    const conn = await buildRouterConnection();

    fetchMock.mockResolvedValueOnce(jsonResponse({}));

    await expect(
      conn.getLatestBlockhashForTransaction(txMessage),
    ).rejects.toThrow(/returned no result/);
  });

  it("prefers error over result when a non-conforming server returns both", async () => {
    const conn = await buildRouterConnection();

    fetchMock.mockResolvedValueOnce(
      jsonResponse({
        jsonrpc: "2.0",
        id: 1,
        result: { blockhash: "abc", lastValidBlockHeight: 100 },
        error: { code: -32000, message: "server error" },
      }),
    );

    await expect(
      conn.getLatestBlockhashForTransaction(txMessage),
    ).rejects.toBeInstanceOf(RouterRpcError);
  });

  it("accepts lastValidBlockHeight === 0 (valid on local validators)", async () => {
    const conn = await buildRouterConnection();

    fetchMock.mockResolvedValueOnce(
      jsonResponse({
        jsonrpc: "2.0",
        id: 1,
        result: { blockhash: "abc", lastValidBlockHeight: 0 },
      }),
    );

    const out = await conn.getLatestBlockhashForTransaction(txMessage);
    expect(out.blockhash).toBe("abc");
    expect(out.lastValidBlockHeight).toBe(0n);
  });

  it("happy path returns bigint lastValidBlockHeight", async () => {
    const conn = await buildRouterConnection();

    fetchMock.mockResolvedValueOnce(
      jsonResponse({
        jsonrpc: "2.0",
        id: 1,
        result: { blockhash: "def", lastValidBlockHeight: 100 },
      }),
    );

    const out = await conn.getLatestBlockhashForTransaction(txMessage);
    expect(out.blockhash).toBe("def");
    expect(out.lastValidBlockHeight).toBe(100n);
    expect(typeof out.lastValidBlockHeight).toBe("bigint");
  });

  it("throws with status on non-2xx HTTP response", async () => {
    const conn = await buildRouterConnection();

    fetchMock.mockResolvedValueOnce(textResponse("upstream unavailable", 503));

    await expect(
      conn.getLatestBlockhashForTransaction(txMessage),
    ).rejects.toThrow(/HTTP 503/);
  });

  it("surfaces JSON-RPC error from non-2xx response as RouterRpcError with httpStatus", async () => {
    // Some providers (e.g. Helius) return HTTP 4xx/5xx with a JSON-RPC
    // error body for unsupported methods. Callers must be able to
    // classify these by `code`, not by string-matching the HTTP message.
    const conn = await buildRouterConnection();

    fetchMock.mockResolvedValueOnce(
      jsonResponse(
        {
          jsonrpc: "2.0",
          id: 1,
          error: { code: -32603, message: "Method not found" },
        },
        404,
      ),
    );

    const call = conn.getLatestBlockhashForTransaction(txMessage);
    await expect(call).rejects.toBeInstanceOf(RouterRpcError);
    await call.catch((err: unknown) => {
      expect(err).toBeInstanceOf(RouterRpcError);
      const routerErr = err as RouterRpcError;
      expect(routerErr.code).toBe(-32603);
      expect(routerErr.method).toBe("getBlockhashForAccounts");
      expect(routerErr.httpStatus).toBe(404);
      expect(routerErr.message).toMatch(/HTTP 404/);
      expect(routerErr.message).toMatch(/Method not found/);
    });
  });

  it("throws plain Error (not RouterRpcError) on non-2xx with non-JSON body", async () => {
    // Guards against over-classification: a bare HTML/text error page
    // from an upstream proxy has no JSON-RPC code to classify on, so it
    // must surface as a plain Error — never silently smuggled as a
    // RouterRpcError that callers might incorrectly treat as
    // "method not found" via the message regex in isRouter.
    const conn = await buildRouterConnection();

    fetchMock.mockResolvedValueOnce(textResponse("<html>500 oops</html>", 500));

    const call = conn.getLatestBlockhashForTransaction(txMessage);
    await expect(call).rejects.toThrow(/HTTP 500/);
    await call.catch((err: unknown) => {
      expect(err).not.toBeInstanceOf(RouterRpcError);
    });
  });
});

describe("isRouter", () => {
  it("returns true on a valid router result", async () => {
    fetchMock.mockResolvedValueOnce(
      jsonResponse({
        jsonrpc: "2.0",
        id: 1,
        result: { blockhash: "abc", lastValidBlockHeight: 1 },
      }),
    );
    await expect(utils.isRouter("http://router.test")).resolves.toBe(true);
  });

  it("returns false on JSON-RPC -32601 (method not found)", async () => {
    fetchMock.mockResolvedValueOnce(
      jsonResponse({
        jsonrpc: "2.0",
        id: 1,
        error: { code: -32601, message: "Method not found" },
      }),
    );
    await expect(utils.isRouter("http://plain-rpc.test")).resolves.toBe(false);
  });

  it("rethrows on transport failure", async () => {
    fetchMock.mockRejectedValueOnce(new Error("ECONNREFUSED"));
    await expect(utils.isRouter("http://unreachable.test")).rejects.toThrow(
      /ECONNREFUSED/,
    );
  });

  it("rethrows on non-2xx HTTP response", async () => {
    fetchMock.mockResolvedValueOnce(textResponse("bad gateway", 502));
    await expect(utils.isRouter("http://bad.test")).rejects.toThrow(/HTTP 502/);
  });

  it("rethrows on unexpected JSON-RPC error codes", async () => {
    fetchMock.mockResolvedValueOnce(
      jsonResponse({
        jsonrpc: "2.0",
        id: 1,
        error: { code: -32000, message: "server error" },
      }),
    );
    await expect(utils.isRouter("http://router.test")).rejects.toThrow(
      /-32000/,
    );
  });

  it("returns false on Helius-style HTTP 404 + -32603 Method not found", async () => {
    // Real devnet Helius response: HTTP 404 with JSON-RPC error body
    // using the non-standard code -32603. The probe must classify this
    // as non-router (return false) rather than rethrow, otherwise
    // Connection.create will blow up for any consumer pointing at a
    // plain L1 RPC that happens to use this provider shape.
    fetchMock.mockResolvedValueOnce(
      jsonResponse(
        {
          jsonrpc: "2.0",
          error: { code: -32603, message: "Method not found" },
        },
        404,
      ),
    );
    await expect(
      utils.isRouter("https://devnet.helius-rpc.com/?api-key=X"),
    ).resolves.toBe(false);
  });

  it("returns false on JSON-RPC error whose message matches /method not found/i with non-standard code on 2xx", async () => {
    // Same classifier, different shape: 2xx response with a JSON-RPC
    // error using an unusual code but the canonical "Method not found"
    // message. The message-based fallback prevents flaky provider
    // classifications from cascading into Connection.create failures.
    fetchMock.mockResolvedValueOnce(
      jsonResponse({
        jsonrpc: "2.0",
        id: 1,
        error: { code: -32099, message: "method not found" },
      }),
    );
    await expect(utils.isRouter("http://weird-rpc.test")).resolves.toBe(false);
  });
});

describe("Connection.create", () => {
  it("propagates transient isRouter failures instead of misclassifying", async () => {
    fetchMock.mockRejectedValueOnce(new Error("ECONNREFUSED"));
    await expect(Connection.create("http://flaky.test")).rejects.toThrow(
      /ECONNREFUSED/,
    );
  });
});

describe("postRouterRpc transport errors via getLatestBlockhashForTransaction", () => {
  it("wraps AbortError (from the 10s timeout) with method context and preserves cause", async () => {
    const conn = await buildRouterConnection();

    const abortErr = new DOMException(
      "The operation was aborted.",
      "AbortError",
    );
    fetchMock.mockRejectedValueOnce(abortErr);

    const call = conn.getLatestBlockhashForTransaction(txMessage);
    await expect(call).rejects.toThrow(
      /Magic Router getBlockhashForAccounts timed out after 10s/,
    );
    await call.catch((err: unknown) => {
      expect((err as Error & { cause?: unknown }).cause).toBe(abortErr);
    });
  });

  it("wraps TimeoutError (newer runtimes) with the same shape as AbortError", async () => {
    const conn = await buildRouterConnection();

    const timeoutErr = new DOMException("Timed out.", "TimeoutError");
    fetchMock.mockRejectedValueOnce(timeoutErr);

    const call = conn.getLatestBlockhashForTransaction(txMessage);
    await expect(call).rejects.toThrow(/timed out after 10s/);
    await call.catch((err: unknown) => {
      expect((err as Error & { cause?: unknown }).cause).toBe(timeoutErr);
    });
  });

  it("propagates generic transport errors (e.g. ECONNREFUSED) unchanged", async () => {
    const conn = await buildRouterConnection();

    fetchMock.mockRejectedValueOnce(new Error("ECONNREFUSED"));

    await expect(
      conn.getLatestBlockhashForTransaction(txMessage),
    ).rejects.toThrow(/ECONNREFUSED/);
  });

  it("attaches text-read error as cause when non-2xx body cannot be read", async () => {
    const conn = await buildRouterConnection();

    const readErr = new Error("stream closed");
    fetchMock.mockResolvedValueOnce({
      ok: false,
      status: 502,
      json: async () => {
        throw new Error("not json");
      },
      text: async () => {
        throw readErr;
      },
    });

    const call = conn.getLatestBlockhashForTransaction(txMessage);
    await expect(call).rejects.toThrow(/HTTP 502/);
    await call.catch((err: unknown) => {
      expect((err as Error & { cause?: unknown }).cause).toBe(readErr);
    });
  });
});

describe("postRouterRpc lenient parsing (via getLatestBlockhashForTransaction)", () => {
  it("falls through to plain HTTP error when non-2xx body has error as a string (not an object)", async () => {
    const conn = await buildRouterConnection();

    fetchMock.mockResolvedValueOnce(
      jsonResponse({ jsonrpc: "2.0", error: "server broken" }, 500),
    );

    const call = conn.getLatestBlockhashForTransaction(txMessage);
    await expect(call).rejects.toThrow(/HTTP 500/);
    await call.catch((err: unknown) => {
      expect(err).not.toBeInstanceOf(RouterRpcError);
    });
  });

  it("falls through to plain HTTP error when non-2xx body has a non-number code (e.g. stringified)", async () => {
    const conn = await buildRouterConnection();

    fetchMock.mockResolvedValueOnce(
      jsonResponse(
        { jsonrpc: "2.0", error: { code: "-32601", message: "Method not found" } },
        500,
      ),
    );

    const call = conn.getLatestBlockhashForTransaction(txMessage);
    await expect(call).rejects.toThrow(/HTTP 500/);
    await call.catch((err: unknown) => {
      expect(err).not.toBeInstanceOf(RouterRpcError);
    });
  });

  it("surfaces RouterRpcError with <no message> when non-2xx body omits message", async () => {
    const conn = await buildRouterConnection();

    fetchMock.mockResolvedValueOnce(
      jsonResponse({ jsonrpc: "2.0", error: { code: -32000 } }, 500),
    );

    const call = conn.getLatestBlockhashForTransaction(txMessage);
    await expect(call).rejects.toBeInstanceOf(RouterRpcError);
    await call.catch((err: unknown) => {
      expect((err as RouterRpcError).code).toBe(-32000);
      expect((err as RouterRpcError).httpStatus).toBe(500);
      expect((err as RouterRpcError).message).toMatch(/<no message>/);
    });
  });

  it("surfaces RouterRpcError with <no message> when 2xx error body omits message", async () => {
    const conn = await buildRouterConnection();

    fetchMock.mockResolvedValueOnce(
      jsonResponse({ jsonrpc: "2.0", id: 1, error: { code: -32000 } }),
    );

    const call = conn.getLatestBlockhashForTransaction(txMessage);
    await expect(call).rejects.toBeInstanceOf(RouterRpcError);
    await call.catch((err: unknown) => {
      expect((err as RouterRpcError).message).toMatch(/<no message>/);
    });
  });

  it("falls through to 'returned no result' when 2xx body has a non-finite error code and no result", async () => {
    const conn = await buildRouterConnection();

    fetchMock.mockResolvedValueOnce(
      jsonResponse({
        jsonrpc: "2.0",
        id: 1,
        error: { code: "-32601", message: "Method not found" },
      }),
    );

    // Symmetric with the non-2xx path: a malformed JSON-RPC error (non-finite
    // code) is not surfaced as a typed RouterRpcError. The caller still sees
    // a meaningful error.
    const call = conn.getLatestBlockhashForTransaction(txMessage);
    await expect(call).rejects.toThrow(/returned no result/);
    await call.catch((err: unknown) => {
      expect(err).not.toBeInstanceOf(RouterRpcError);
    });
  });

  it("accepts 2xx response with result: null", async () => {
    // JSON-RPC spec permits `result: null` as a valid success value. The
    // generic `postRouterRpc<T>` helper must not treat it as an error — that
    // would block future callers where `null` is a legitimate response.
    fetchMock.mockResolvedValueOnce(
      jsonResponse({ jsonrpc: "2.0", id: 1, result: null }),
    );

    await expect(
      postRouterRpc<null>("http://router.test", "someMethod", []),
    ).resolves.toBeNull();
  });
});
