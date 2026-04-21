import { AccountRole, Signature, TransactionMessage } from "@solana/kit";
import { postRouterRpc, RouterRpcError } from "./router-rpc";

/**
 * Extracts all writable accounts from a transaction message.
 *
 * @param transactionMessage - The transaction message to analyze.
 * @returns An array of writable account addresses.
 */
export function getWritableAccounts(
  transactionMessage: TransactionMessage,
): string[] {
  const writableAccounts = new Set<string>();
  for (const instruction of transactionMessage.instructions) {
    if (instruction.accounts) {
      for (const account of instruction.accounts) {
        if (
          account.role === AccountRole.WRITABLE ||
          account.role === AccountRole.WRITABLE_SIGNER
        ) {
          writableAccounts.add(account.address.toString());
        }
      }
    }
  }
  return Array.from(writableAccounts);
}

/**
 * Probes a Solana RPC endpoint to determine whether it implements Magic
 * Router methods by calling `getBlockhashForAccounts` with an empty account
 * set.
 *
 * Classifies the endpoint as non-router when the probe surfaces a
 * {@link RouterRpcError} that looks semantically like "method not found":
 *   - the standard JSON-RPC code `-32601`, OR
 *   - any code whose message matches `/method not found/i` (covers
 *     providers like Helius that return the same semantic error under a
 *     non-standard code such as `-32603`, sometimes on a non-2xx HTTP
 *     response).
 *
 * All other failures — transport errors, HTTP errors whose body isn't a
 * parseable JSON-RPC error, unexpected JSON-RPC codes without the
 * method-not-found message — rethrow.
 *
 * Non-router classification is a latch: once set at construction, the
 * `Connection` never re-probes. A transient network or server failure during
 * the probe must therefore propagate rather than collapse into `false`,
 * otherwise the `Connection` would be permanently mis-classified for its
 * lifetime.
 *
 * @param clusterUrlHttp - The HTTP RPC endpoint to probe.
 * @returns `true` if the endpoint responds with a valid router result,
 *          `false` if the method is unsupported.
 * @throws {RouterRpcError} If the endpoint returns a JSON-RPC error that
 *                          is neither `-32601` nor carries a "method not
 *                          found" message.
 * @throws {Error} On transport failures or HTTP failures whose body cannot
 *                 be classified as a JSON-RPC error.
 */
export async function isRouter(clusterUrlHttp: string): Promise<boolean> {
  try {
    const result = await postRouterRpc<{
      blockhash: string;
      lastValidBlockHeight: number;
    }>(clusterUrlHttp, "getBlockhashForAccounts", [[]]);
    return typeof result.blockhash === "string" && result.blockhash.length > 0;
  } catch (err) {
    if (
      err instanceof RouterRpcError &&
      (err.code === -32601 || /method not found/i.test(err.message))
    ) {
      return false;
    }
    throw err;
  }
}

/**
 * Parses log messages from a scheduling transaction to extract the scheduled commit signature.
 *
 * This function looks for log messages containing the prefix `"ScheduledCommitSent signature: "`
 * and returns the signature following it.
 *
 * @param logMessages - An array of log messages from a transaction's meta.
 * @returns The extracted `Signature` if found, or `null` if no matching log message exists.
 */
export function parseScheduleCommitsLogsMessage(
  logMessages: readonly string[],
): Signature | null {
  for (const message of logMessages) {
    const signaturePrefix = "ScheduledCommitSent signature: ";
    if (message.includes(signaturePrefix)) {
      return message.split(signaturePrefix)[1] as Signature;
    }
  }
  return null;
}

/**
 * Parses log messages from a commit transaction to extract the commitment signature.
 *
 * This function looks for log messages containing the prefix `"ScheduledCommitSent signature[0]: "`
 * and returns the signature following it.
 *
 * @param logMessages - An array of log messages from a transaction's meta.
 * @returns The extracted `Signature` if found, or `null` if no matching log message exists.
 */
export function parseCommitsLogsMessage(
  logMessages: readonly string[],
): Signature | null {
  for (const message of logMessages) {
    const signaturePrefix = "ScheduledCommitSent signature[0]: ";
    if (message.includes(signaturePrefix)) {
      return message.split(signaturePrefix)[1] as Signature;
    }
  }
  return null;
}
