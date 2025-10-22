import { AccountRole, Signature, TransactionMessage } from "@solana/kit";

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
 * Checks whether a given Solana RPC endpoint supports router methods.
 *
 * @param clusterUrlHttp - The HTTP endpoint to test.
 * @returns `true` if router support is detected, otherwise `false`.
 */
export async function isRouter(clusterUrlHttp: string): Promise<boolean> {
  const response = await fetch(clusterUrlHttp, {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({
      jsonrpc: "2.0",
      id: 1,
      method: "getBlockhashForAccounts",
      params: [[]],
    }),
  });

  const { result } = (await response.json()) as {
    result: { blockhash: string; lastValidBlockHeight: number };
  };

  return (
    result != null &&
    typeof result.blockhash === "string" &&
    result.blockhash.length > 0
  );
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
