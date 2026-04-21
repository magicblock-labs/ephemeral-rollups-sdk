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
  /** JSON-RPC method that produced the error. */
  public readonly method: string;
  /** JSON-RPC error code returned by the Magic Router. */
  public readonly code: number;
  /** Optional `data` payload attached to the JSON-RPC error, if any. */
  public readonly data?: unknown;
  /** HTTP status code, set only when the error body rode on a non-2xx response. */
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
 * Attempts to extract a well-formed JSON-RPC error object from a response
 * body. Returns `null` if the body isn't JSON at all, isn't an object, has
 * no `error` field, or has an `error` field whose `code` isn't a finite
 * number. Intentionally lenient about the `message` field — some providers
 * omit or mistype it — since the code is the load-bearing classifier.
 */
function parseJsonRpcError(bodyText: string): {
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

/**
 * POSTs a JSON-RPC request to a Magic Router endpoint and returns the parsed
 * `result`. Surfaces structured JSON-RPC errors as {@link RouterRpcError}
 * rather than silently swallowing them, so callers can classify failures by
 * `code` and see the upstream message.
 *
 * Parses JSON-RPC error bodies on BOTH 2xx and non-2xx responses: some
 * providers (notably Helius) return HTTP 404 or 500 with a JSON-RPC error
 * body for unsupported methods. Short-circuiting on `!res.ok` before
 * parsing would force callers to string-match the generic HTTP error
 * message instead of classifying by `code`. When the error body rode on a
 * non-2xx response, the thrown {@link RouterRpcError} carries the HTTP
 * status in `httpStatus`.
 *
 * Requests are bounded by a 10-second timeout to avoid hanging on routers
 * that accept TCP but never respond.
 *
 * @typeParam T - The shape of the `result` field expected from the router.
 * @param url - The HTTP endpoint of the Magic Router.
 * @param method - The JSON-RPC method name to invoke.
 * @param params - The positional parameters to pass to the method.
 * @returns The decoded `result` payload of the JSON-RPC response.
 * @throws {RouterRpcError} If the response body contains a well-formed
 *                          JSON-RPC `error` (regardless of HTTP status).
 * @throws {Error} If a non-2xx response had no parseable JSON-RPC error
 *                 body, if the 2xx body cannot be parsed as JSON, or if a
 *                 2xx body has neither `result` nor `error`.
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
      // Losing the body is acceptable; the status code is the load-bearing
      // signal and is already in the message.
      bodyText = "<unreadable body>";
    }
    const rpcError = parseJsonRpcError(bodyText);
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
  let body: {
    result?: T;
    error?: { code: number; message: string; data?: unknown };
  };
  try {
    body = (await res.json()) as typeof body;
  } catch (err) {
    const wrapped = new Error(
      `Magic Router ${method} returned non-JSON body: ${(err as Error).message}`,
    );
    (wrapped as Error & { cause?: unknown }).cause = err;
    throw wrapped;
  }
  if (body.error != null) {
    const { code, message, data } = body.error;
    if (typeof code !== "number" || !Number.isFinite(code)) {
      throw new Error(
        `Magic Router ${method} returned malformed error body: ${JSON.stringify(body.error)}`,
      );
    }
    throw new RouterRpcError({
      method,
      code,
      message: message ?? "<no message>",
      data,
    });
  }
  if (body.result == null) {
    throw new Error(
      `Magic Router ${method} returned no result: ${JSON.stringify(body)}`,
    );
  }
  return body.result;
}
