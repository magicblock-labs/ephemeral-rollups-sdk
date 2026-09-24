const ROUTER_RPC_TIMEOUT_MS = 10_000;

/** Structured JSON-RPC error from a Magic Router response. */
export class RouterRpcError extends Error {
  public readonly method: string;
  public readonly code: number;
  public readonly data?: unknown;
  /** Set only when the JSON-RPC error body rode on a non-2xx HTTP response. */
  public readonly httpStatus?: number;

  constructor(args: {
    method: string;
    code: number;
    message: string;
    data?: unknown;
    httpStatus?: number;
  }) {
    const statusPrefix =
      args.httpStatus != null ? `HTTP ${args.httpStatus} ` : "";
    super(
      `Magic Router ${args.method} ${statusPrefix}error ${args.code}: ${args.message}`,
    );
    this.name = "RouterRpcError";
    this.method = args.method;
    this.code = args.code;
    this.data = args.data;
    this.httpStatus = args.httpStatus;
  }
}

/** Lenient on `message`, strict on `code` — `code` is the load-bearing classifier. */
function parseJsonRpcErrorFromObject(parsed: unknown): {
  code: number;
  message: string;
  data?: unknown;
} | null {
  if (parsed == null || typeof parsed !== "object") {
    return null;
  }
  const err = (parsed as { error?: unknown }).error;
  if (err == null || typeof err !== "object") {
    return null;
  }
  const { code, message, data } = err as {
    code?: unknown;
    message?: unknown;
    data?: unknown;
  };
  if (typeof code !== "number" || !Number.isFinite(code)) {
    return null;
  }
  return {
    code,
    message: typeof message === "string" ? message : "<no message>",
    data,
  };
}

function parseJsonRpcErrorFromText(bodyText: string): {
  code: number;
  message: string;
  data?: unknown;
} | null {
  let parsed: unknown;
  try {
    parsed = JSON.parse(bodyText);
  } catch {
    return null;
  }
  return parseJsonRpcErrorFromObject(parsed);
}

/** Providers like Helius return HTTP 4xx/5xx with a JSON-RPC error body for unsupported methods, so we parse error bodies even when !res.ok. */
export async function postRouterRpc<T>(
  url: string,
  method: string,
  params: unknown[],
): Promise<T> {
  let res: Response;
  try {
    res = await fetch(url, {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({ jsonrpc: "2.0", id: 1, method, params }),
      signal: AbortSignal.timeout(ROUTER_RPC_TIMEOUT_MS),
    });
  } catch (err) {
    const name = (err as { name?: unknown })?.name;
    if (name === "AbortError" || name === "TimeoutError") {
      throw new Error(
        `Magic Router ${method} timed out after ${ROUTER_RPC_TIMEOUT_MS / 1000}s`,
        { cause: err },
      );
    }
    throw err;
  }
  if (!res.ok) {
    let bodyText: string;
    let readErr: unknown;
    try {
      bodyText = await res.text();
    } catch (err) {
      readErr = err;
      bodyText = "<unreadable body>";
    }
    const rpcError = parseJsonRpcErrorFromText(bodyText);
    if (rpcError != null) {
      throw new RouterRpcError({
        method,
        code: rpcError.code,
        message: rpcError.message,
        data: rpcError.data,
        httpStatus: res.status,
      });
    }
    throw new Error(
      `Magic Router ${method} HTTP ${res.status}: ${bodyText.slice(0, 2048)}`,
      readErr != null ? { cause: readErr } : undefined,
    );
  }
  let body: unknown;
  try {
    body = await res.json();
  } catch (err) {
    const message = err instanceof Error ? err.message : String(err);
    throw new Error(
      `Magic Router ${method} returned non-JSON body: ${message}`,
      { cause: err },
    );
  }
  const rpcError = parseJsonRpcErrorFromObject(body);
  if (rpcError != null) {
    throw new RouterRpcError({
      method,
      code: rpcError.code,
      message: rpcError.message,
      data: rpcError.data,
    });
  }
  if (
    body == null ||
    typeof body !== "object" ||
    !Object.prototype.hasOwnProperty.call(body, "result")
  ) {
    throw new Error(
      `Magic Router ${method} returned no result: ${JSON.stringify(body)}`,
    );
  }
  return (body as { result: T }).result;
}
