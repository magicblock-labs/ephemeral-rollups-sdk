/**
 * Error thrown when a Magic Router responds with a structured JSON-RPC
 * `error` body. Exposes the upstream `method`, `code`, and optional `data`
 * so callers can classify failures without parsing the error message string.
 *
 * When the JSON-RPC error body rode on a non-2xx HTTP response (as some RPC
 * providers, e.g. Helius, return for unsupported methods), `httpStatus`
 * records the status code so callers can distinguish "2xx with JSON-RPC
 * error" from "4xx/5xx with JSON-RPC error" if they need to.
 */
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

/**
 * Extracts a well-formed JSON-RPC error from a parsed body. Lenient on
 * `message` (defaulted when missing), strict on `code` (must be a finite
 * number) since `code` is the load-bearing classifier.
 */
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

/**
 * POSTs a JSON-RPC request to a Magic Router endpoint and returns the parsed
 * `result`. Parses JSON-RPC error bodies on both 2xx and non-2xx responses
 * because providers such as Helius return HTTP 4xx/5xx with a structured
 * JSON-RPC error body for unsupported methods — callers must be able to
 * classify by `code` instead of string-matching HTTP messages. Bounded by a
 * 10-second timeout.
 */
export async function postRouterRpc<T>(
  url: string,
  method: string,
  params: unknown[],
): Promise<T> {
  const res = await fetch(url, {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({ jsonrpc: "2.0", id: 1, method, params }),
    signal: AbortSignal.timeout(10_000),
  });
  if (!res.ok) {
    let bodyText: string;
    try {
      bodyText = await res.text();
    } catch {
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
    );
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
