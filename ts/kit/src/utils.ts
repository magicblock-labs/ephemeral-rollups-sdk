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
 * Result is latched by Connection; transient probe failures must propagate, not collapse to `false`.
 * Also matches `/method not found/i` to classify providers like Helius that use `-32603` for this case.
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
