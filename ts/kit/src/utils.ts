import { AccountRole, Signature, TransactionMessage } from "@solana/kit";

const OUTBOX_PROGRAM_ID = "outboxintent1111111111111111111111111111111";

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

export interface SettlementReceipt {
  intentAddress: string | null;
  result: { errorMessage: string | null; logs: string[] };
  signatures: string[];
}

export function parseOutboxAddresses(logMessages: readonly string[]): string[] {
  const prefix = "ScheduledCommitSent outbox: ";
  return logMessages.flatMap((line) => {
    const i = line.indexOf(prefix);
    return i < 0 ? [] : [line.slice(i + prefix.length).trim()];
  });
}

export function receiptFromLogs(
  intentAddress: string | null,
  logMessages: readonly string[],
): SettlementReceipt {
  const logs = [...logMessages];
  const signatures = logs.flatMap((line) => {
    const match = line.match(/ScheduledCommitSent signature\[\d+\]: (.+)$/);
    return match ? [match[1].trim()] : [];
  });
  const errorLine = logs.find((line) =>
    line.includes("ScheduledCommitSent error message:"),
  );
  return {
    intentAddress,
    result: {
      errorMessage:
        errorLine?.split("ScheduledCommitSent error message:")[1]?.trim() ??
        null,
      logs,
    },
    signatures,
  };
}

export function hasAuthorizedCompletion(
  tx: unknown,
  validator: string,
  outbox: string,
): boolean {
  const transaction = tx as {
    transaction?: {
      message?: {
        accountKeys?: Array<string | { pubkey: string; signer?: boolean }>;
        staticAccountKeys?: string[];
        header?: { numRequiredSignatures?: number };
        instructions?: Array<{
          programId?: string;
          programIdIndex?: number;
          accounts?: Array<string | number>;
        }>;
        compiledInstructions?: Array<{
          programIdIndex: number;
          accountKeyIndexes: number[];
        }>;
      };
    };
  };
  const message = transaction.transaction?.message;
  if (!message) return false;
  const keys =
    message.staticAccountKeys ??
    message.accountKeys?.map((key) =>
      typeof key === "string" ? key : key.pubkey,
    ) ??
    [];
  const signers = message.header?.numRequiredSignatures ?? 0;
  if (!keys.slice(0, signers).includes(validator)) return false;
  const instructions =
    message.compiledInstructions ?? message.instructions ?? [];
  return instructions.some((ix) => {
    const program =
      ("programId" in ix ? ix.programId : undefined) ??
      keys[ix.programIdIndex ?? -1];
    const accounts =
      "accountKeyIndexes" in ix
        ? ix.accountKeyIndexes.map((index) => keys[index])
        : (ix.accounts ?? []).map((account) =>
            typeof account === "number" ? keys[account] : account,
          );
    return program === OUTBOX_PROGRAM_ID && accounts.includes(outbox);
  });
}
