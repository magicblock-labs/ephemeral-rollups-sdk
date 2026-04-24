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
    cause?: unknown;
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
    // Forward `cause` without requiring the `es2022.error` lib on the
    // `Error` constructor signature. Runtime support is Node 16.9+.
    if (args.cause != null) {
      (this as Error & { cause?: unknown }).cause = args.cause;
    }
    // Restore prototype chain so `instanceof RouterRpcError` survives
    // downlevel emit to ES2015-style `extends Error`.
    Object.setPrototypeOf(this, new.target.prototype);
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

/** Parses JSON-RPC errors on both 2xx and non-2xx — providers like Helius return HTTP 4xx/5xx with an RPC error body for unsupported methods. */
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
      signal: AbortSignal.timeout(10_000),
    });
  } catch (err) {
    // AbortSignal.timeout surfaces as `AbortError` on older Node, `TimeoutError` on newer.
    const name = (err as { name?: unknown })?.name;
    if (name === "AbortError" || name === "TimeoutError") {
      const wrapped = new Error(
        `Magic Router ${method} timed out after 10s`,
      );
      (wrapped as Error & { cause?: unknown }).cause = err;
      throw wrapped;
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
    const httpError = new Error(
      `Magic Router ${method} HTTP ${res.status}: ${bodyText.slice(0, 2048)}`,
    );
    if (readErr != null) {
      (httpError as Error & { cause?: unknown }).cause = readErr;
    }
    throw httpError;
  }
  let body: unknown;
  try {
    body = await res.json();
  } catch (err) {
    const wrapped = new Error(
      `Magic Router ${method} returned non-JSON body: ${(err as Error).message}`,
    );
    (wrapped as Error & { cause?: unknown }).cause = err;
    throw wrapped;
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
