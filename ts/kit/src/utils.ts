import { AccountRole, Signature, TransactionMessage } from "@solana/kit";
import bs58 from "bs58";

const OUTBOX_PROGRAM_ID = "outboxintent1111111111111111111111111111111";
const SCHEDULED_COMMIT_SENT_TAG = 2;

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

export interface ReceiptWaitOptions {
  timeoutMs?: number;
  pollIntervalMs?: number;
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

export function logsFromProgram(
  logs: readonly string[],
  programId: string,
): string[] {
  const stack: string[] = [];
  const programLogs: string[] = [];
  for (const line of logs) {
    const invoke = line.match(/^Program (\S+) invoke \[\d+\]$/);
    if (invoke) {
      stack.push(invoke[1]);
      continue;
    }
    const end = line.match(/^Program (\S+) (?:success|failed:.*)$/);
    if (end) {
      const index = stack.lastIndexOf(end[1]);
      if (index >= 0) stack.length = index;
      continue;
    }
    if (
      stack[stack.length - 1] === programId &&
      line.startsWith("Program log: ")
    )
      programLogs.push(line);
  }
  return programLogs;
}

export function receiptFromCompletion(
  tx: unknown,
  outbox: string,
): SettlementReceipt | null {
  if (tx == null || typeof tx !== "object") return null;
  const transaction = tx as {
    transaction?: {
      message?: {
        instructions?: Array<{
          programId?: string;
          accounts?: string[];
          data?: string;
        }>;
      };
    };
    meta?: {
      err?: unknown;
      logMessages?: string[] | null;
    } | null;
  };
  const message = transaction.transaction?.message;
  const meta = transaction.meta;
  if (!message || meta == null || meta.err !== null) return null;
  // A successful Outbox instruction has already checked the validator signer and PDA.
  const logs = logsFromProgram(meta.logMessages ?? [], OUTBOX_PROGRAM_ID);
  const groups: string[][] = [];
  for (const line of logs) {
    if (/^Program log: ScheduledCommitSent id: \d+,/.test(line))
      groups.push([line]);
    else if (groups.length > 0) groups[groups.length - 1].push(line);
  }

  for (const group of groups) {
    const idMatch = group[0]?.match(/id: (\d+),/);
    if (!idMatch) continue;
    const intentId = BigInt(idMatch[1]);
    const isCompletion = (message.instructions ?? []).some((instruction) => {
      if (
        instruction.programId !== OUTBOX_PROGRAM_ID ||
        instruction.accounts?.[1] !== outbox ||
        instruction.data == null
      )
        return false;
      let data: Uint8Array;
      try {
        data = bs58.decode(instruction.data);
      } catch {
        return false;
      }
      if (
        data.length < 12 ||
        // wincode encodes this enum tag as u32, followed by message_id as u64.
        data[0] !== SCHEDULED_COMMIT_SENT_TAG ||
        data[1] !== 0 ||
        data[2] !== 0 ||
        data[3] !== 0
      )
        return false;
      let instructionId = 0n;
      for (let byte = 7; byte >= 0; byte--)
        instructionId = (instructionId << 8n) | BigInt(data[4 + byte]);
      return instructionId === intentId;
    });
    if (isCompletion) return receiptFromLogs(outbox, group);
  }
  return null;
}
