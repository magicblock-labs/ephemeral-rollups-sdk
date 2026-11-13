// eslint-disable-next-line @typescript-eslint/consistent-type-imports
import { Connection, PublicKey } from "@solana/web3.js";
import bs58 from "bs58";
import { MAGIC_PROGRAM_ID } from "./constants";

/**
 * Get the signature of the accounts commitment transaction to the base layer
 * @param transactionSignature
 * @param ephemeralConnection
 * @param transactionLookupTimeoutMs Maximum time to wait for either transaction lookup (default: 30 seconds).
 * @constructor
 */
export async function GetCommitmentSignature(
  transactionSignature: string,
  ephemeralConnection: Connection,
  transactionLookupTimeoutMs = 30_000,
): Promise<string> {
  const txSchedulingSgn = await getTransactionWithRetry(
    async () =>
      ephemeralConnection.getTransaction(transactionSignature, {
        maxSupportedTransactionVersion: 0,
      }),
    transactionLookupTimeoutMs,
  );
  console.log("> txSchedulingSgn: ", txSchedulingSgn);

  if (txSchedulingSgn?.meta == null) {
    throw new Error("Transaction not found or meta is null");
  }
  // A later instruction can fail after magic-program logs ScheduledCommitSent.
  // In that case the overall transaction fails and nothing was scheduled, so
  // we must not return a signature that will never land on the ER.
  if (txSchedulingSgn.meta.err !== null) {
    throw new Error(
      "Transaction failed; commitment was not scheduled (ScheduledCommitSent logs from earlier instructions are not reliable when the transaction errors)",
    );
  }

  const schedulingLogs = logsFromProgram(
    txSchedulingSgn.meta.logMessages ?? [],
    MAGIC_PROGRAM_ID.toBase58(),
  );
  const scheduledCommitSgn = parseScheduleCommitsLogsMessage(schedulingLogs);
  if (scheduledCommitSgn == null) {
    const outboxAddresses = parseOutboxAddresses(schedulingLogs);
    if (outboxAddresses.length === 1) {
      const receipt = await waitForIntent(
        outboxAddresses[0],
        ephemeralConnection,
      );
      if (receipt.signatures.length === 0)
        throw new Error("Settlement receipt contains no base-chain signature");
      return receipt.signatures[0];
    }
    if (outboxAddresses.length > 1)
      throw new Error(
        "Scheduling transaction created multiple intents; use waitForSettlement",
      );
    throw new Error("ScheduledCommitSent signature not found");
  }

  const latestBlockhash = await ephemeralConnection.getLatestBlockhash();
  await ephemeralConnection.confirmTransaction({
    signature: scheduledCommitSgn,
    ...latestBlockhash,
  });

  const txCommitInfo = await getTransactionWithRetry(
    async () =>
      ephemeralConnection.getTransaction(scheduledCommitSgn, {
        maxSupportedTransactionVersion: 0,
      }),
    transactionLookupTimeoutMs,
  );
  if (txCommitInfo?.meta == null) {
    throw new Error("Transaction not found or meta is null");
  }
  if (txCommitInfo.meta.err !== null) {
    throw new Error(
      "Scheduled commit transaction failed; commitment signature is not available",
    );
  }

  const commitSignature = parseCommitsLogsMessage(
    logsFromProgram(
      txCommitInfo.meta.logMessages ?? [],
      MAGIC_PROGRAM_ID.toBase58(),
    ),
  );
  if (commitSignature == null) {
    throw new Error("Unable to find Commitment signature");
  }

  return commitSignature;
}

export interface SettlementReceipt {
  /** Outbox PDA for outbox receipts; null for legacy receipts. */
  outboxAddress: string | null;
  /** @deprecated Use outboxAddress instead. */
  intentAddress: string | null;
  result: { errorMessage: string | null; logs: string[] };
  signatures: string[];
}

export interface ReceiptWaitOptions {
  timeoutMs?: number;
  pollIntervalMs?: number;
}
const OUTBOX_PREFIX = "ScheduledCommitSent outbox: ";
const OUTBOX_PROGRAM_ID = "outboxintent1111111111111111111111111111111";
const OUTBOX_PROGRAM = new PublicKey(OUTBOX_PROGRAM_ID);
const SCHEDULED_COMMIT_SENT_TAG = 2;

async function withDeadline<T>(
  promise: Promise<T>,
  deadline: number,
  timeoutMessage = "Timed out waiting for settlement receipt",
): Promise<T> {
  const remaining = deadline - Date.now();
  if (remaining <= 0) {
    void promise.catch(() => undefined);
    return Promise.reject(new Error(timeoutMessage));
  }
  return new Promise<T>((resolve, reject) => {
    const cleanup = () => {
      clearTimeout(timer);
    };
    const timer = setTimeout(() => {
      cleanup();
      reject(new Error(timeoutMessage));
    }, remaining);
    promise.then(
      (value) => {
        cleanup();
        resolve(value);
      },
      (error) => {
        cleanup();
        reject(error);
      },
    );
  });
}

const TRANSACTION_LOOKUP_POLL_INTERVAL_MS = 1_000;
const TRANSACTION_LOOKUP_TIMEOUT_MESSAGE =
  "Timed out waiting for transaction lookup";

async function getTransactionWithRetry<T>(
  getTransaction: () => Promise<T | null>,
  timeoutMs: number,
): Promise<T> {
  if (!Number.isFinite(timeoutMs) || timeoutMs < 0) {
    throw new RangeError(
      "Transaction lookup timeout must be a non-negative number",
    );
  }

  const deadline = Date.now() + timeoutMs;
  while (true) {
    const transaction = await withDeadline(
      getTransaction(),
      deadline,
      TRANSACTION_LOOKUP_TIMEOUT_MESSAGE,
    );
    if (transaction !== null) return transaction;

    const remaining = deadline - Date.now();
    if (remaining <= 0) {
      throw new Error(`Transaction not found after waiting ${timeoutMs}ms`);
    }
    await new Promise((resolve) =>
      setTimeout(
        resolve,
        Math.min(TRANSACTION_LOOKUP_POLL_INTERVAL_MS, remaining),
      ),
    );
    if (Date.now() >= deadline) {
      throw new Error(`Transaction not found after waiting ${timeoutMs}ms`);
    }
  }
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
      (line.startsWith("Program log: ") || !line.startsWith("Program "))
    )
      programLogs.push(line);
  }
  return programLogs;
}

export function parseOutboxAddresses(logs: readonly string[]): string[] {
  return logs.flatMap((line) => {
    const i = line.indexOf(OUTBOX_PREFIX);
    return i < 0 ? [] : [line.slice(i + OUTBOX_PREFIX.length).trim()];
  });
}

/** Wait for every settlement scheduled by a successful ER transaction. */
export async function waitForSettlement(
  schedulingSignature: string,
  connection: Connection,
  options: ReceiptWaitOptions = {},
): Promise<SettlementReceipt[]> {
  const tx = await connection.getTransaction(schedulingSignature, {
    maxSupportedTransactionVersion: 0,
  });
  if (!tx?.meta) throw new Error("Scheduling transaction not found");
  if (tx.meta.err !== null)
    throw new Error(
      "Scheduling transaction failed; no settlement receipt exists",
    );
  const logs = logsFromProgram(
    tx.meta.logMessages ?? [],
    MAGIC_PROGRAM_ID.toBase58(),
  );
  const outboxAddresses = parseOutboxAddresses(logs);
  if (outboxAddresses.length > 0)
    return Promise.all(
      outboxAddresses.map(async (outboxAddress) =>
        waitForIntent(outboxAddress, connection, options),
      ),
    );
  const legacy = parseScheduleCommitsLogsMessage(logs);
  if (legacy == null)
    throw new Error("No settlement receipt found in scheduling logs");
  await connection.confirmTransaction(legacy);
  return [await readLegacyReceipt(legacy, connection)];
}

/** Wait for a settlement completion recorded in an outbox address's history. */
export async function waitForIntent(
  outboxAddress: string,
  connection: Connection,
  options: ReceiptWaitOptions = {},
): Promise<SettlementReceipt> {
  let outboxPublicKey: PublicKey;
  try {
    outboxPublicKey = new PublicKey(outboxAddress);
  } catch {
    throw new Error(`Invalid settlement outbox address: ${outboxAddress}`);
  }
  const timeoutMs = options.timeoutMs ?? 120_000;
  const pollIntervalMs = options.pollIntervalMs ?? 1_000;
  const deadline = Date.now() + timeoutMs;
  const inspected = new Set<string>();
  let pollCount = 0;
  let previousSignatureList: string | null = null;
  console.log("[waitForIntent] started", { outboxAddress });

  while (Date.now() <= deadline) {
    pollCount++;
    let signatures;
    try {
      signatures = await withDeadline(
        connection.getSignaturesForAddress(outboxPublicKey, { limit: 1000 }),
        deadline,
      );
    } catch (error) {
      console.error("[waitForIntent] getSignaturesForAddress failed", {
        outboxAddress,
        pollCount,
        error,
      });
      throw error;
    }
    const signatureList = signatures.map((item) => item.signature);
    const signatureListKey = signatureList.join(",");
    if (signatureListKey !== previousSignatureList) {
      console.log("[waitForIntent] getSignaturesForAddress completed", {
        outboxAddress,
        pollCount,
        signatureCount: signatures.length,
        signatures: signatureList,
      });
      previousSignatureList = signatureListKey;
    }

    for (const item of signatures) {
      if (inspected.has(item.signature)) continue;
      let tx;
      try {
        tx = await withDeadline(
          connection.getParsedTransaction(item.signature, {
            maxSupportedTransactionVersion: 0,
          }),
          deadline,
        );
      } catch (error) {
        console.error("[waitForIntent] getParsedTransaction failed", {
          outboxAddress,
          signature: item.signature,
          error,
        });
        if (Date.now() >= deadline) throw error;
        continue;
      }
      if (tx?.meta != null) inspected.add(item.signature);
      const transactionLogs = tx?.meta?.logMessages ?? [];
      console.log("[waitForIntent] getParsedTransaction completed", {
        outboxAddress,
        signature: item.signature,
        transactionFound: tx != null,
        metaAvailable: tx?.meta != null,
        transactionError: tx?.meta?.err,
        ...(transactionLogs.length > 0 ? { transactionLogs } : {}),
      });
      if (tx?.meta?.err === null) {
        const receipt = receiptFromCompletion(tx, outboxPublicKey);
        console.log("[waitForIntent] receipt parsing completed", {
          outboxAddress,
          signature: item.signature,
          matched: receipt != null,
          receiptSignatureCount: receipt?.signatures.length ?? 0,
          receiptSignatures: receipt?.signatures ?? [],
        });
        if (receipt != null) return receipt;
      }
    }
    await new Promise<void>((resolve) =>
      setTimeout(
        resolve,
        Math.min(pollIntervalMs, Math.max(0, deadline - Date.now())),
      ),
    );
  }
  console.warn("[waitForIntent] timed out without finding a receipt", {
    outboxAddress,
    pollCount,
    inspectedTransactionCount: inspected.size,
  });
  throw new Error(
    `Timed out waiting for settlement outbox ${outboxAddress}; transaction history may be unavailable`,
  );
}

export function receiptFromCompletion(
  tx: unknown,
  outboxAddress: PublicKey,
): SettlementReceipt | null {
  if (tx == null || typeof tx !== "object") return null;
  const confirmedTx = tx as {
    transaction?: {
      message?: {
        instructions?: Array<{
          programId?: PublicKey | string;
          accounts?: PublicKey[];
          data?: string;
        }>;
      };
    };
    meta?: {
      err?: unknown;
      logMessages?: string[] | null;
      loadedAddresses?: {
        writable?: Array<PublicKey | string>;
        readonly?: Array<PublicKey | string>;
      } | null;
    } | null;
  };
  const message = confirmedTx.transaction?.message;
  const meta = confirmedTx.meta;
  if (!message || meta == null || meta.err !== null) return null;
  // A successful Outbox instruction has already checked the validator signer and PDA.
  const logs = logsFromProgram(meta.logMessages ?? [], OUTBOX_PROGRAM_ID);
  const groups: string[][] = [];
  for (const line of logs) {
    if (/^(?:Program log: )?ScheduledCommitSent id: \d+,/.test(line))
      groups.push([line]);
    else if (groups.length > 0) groups[groups.length - 1].push(line);
  }

  for (const group of groups) {
    const idMatch = group[0]?.match(/id: (\d+),/);
    if (!idMatch) continue;
    const intentId = BigInt(idMatch[1]);
    const isCompletion = (message.instructions ?? []).some((instruction) => {
      if (
        !(instruction.programId instanceof PublicKey) ||
        !instruction.programId.equals(OUTBOX_PROGRAM) ||
        instruction.accounts?.[1]?.equals(outboxAddress) !== true ||
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
    if (isCompletion) return receiptFromLogs(outboxAddress.toBase58(), group);
  }
  return null;
}

async function readLegacyReceipt(
  signature: string,
  connection: Connection,
): Promise<SettlementReceipt> {
  const tx = await connection.getTransaction(signature, {
    maxSupportedTransactionVersion: 0,
  });
  if (!tx?.meta) throw new Error("Legacy settlement notification not found");
  if (tx.meta.err !== null)
    throw new Error("Legacy settlement notification failed");
  return receiptFromLogs(
    null,
    logsFromProgram(tx.meta.logMessages ?? [], MAGIC_PROGRAM_ID.toBase58()),
  );
}

export function receiptFromLogs(
  outboxAddress: string | null,
  logs: string[],
): SettlementReceipt {
  const signatures = logs.flatMap((line) => {
    const match = line.match(/ScheduledCommitSent signature\[\d+\]: (.+)$/);
    return match ? [match[1].trim()] : [];
  });
  const errorLine = logs.find((line) =>
    line.includes("ScheduledCommitSent error message:"),
  );
  return {
    outboxAddress,
    intentAddress: outboxAddress,
    result: {
      errorMessage:
        errorLine?.split("ScheduledCommitSent error message:")[1]?.trim() ??
        null,
      logs,
    },
    signatures,
  };
}

export function parseScheduleCommitsLogsMessage(
  logMessages: string[],
): string | null {
  for (const message of logMessages) {
    const signaturePrefix = "ScheduledCommitSent signature: ";
    if (message.includes(signaturePrefix)) {
      return message.split(signaturePrefix)[1];
    }
  }
  return null;
}

export function parseCommitsLogsMessage(logMessages: string[]): string | null {
  for (const message of logMessages) {
    const signaturePrefix = "ScheduledCommitSent signature[0]: ";
    if (message.includes(signaturePrefix)) {
      return message.split(signaturePrefix)[1];
    }
  }
  return null;
}
