// eslint-disable-next-line @typescript-eslint/consistent-type-imports
import { Connection, PublicKey } from "@solana/web3.js";

/**
 * Get the signature of the accounts commitment transaction to the base layer
 * @param transactionSignature
 * @param ephemeralConnection
 * @constructor
 */
export async function GetCommitmentSignature(
  transactionSignature: string,
  ephemeralConnection: Connection,
): Promise<string> {
  const txSchedulingSgn = await ephemeralConnection.getTransaction(
    transactionSignature,
    { maxSupportedTransactionVersion: 0 },
  );
  if (txSchedulingSgn?.meta == null) {
    throw new Error("Transaction not found or meta is null");
  }
  // A later instruction can fail after magic-program logs ScheduledCommitSent.
  // In that case the overall transaction fails and nothing was scheduled, so
  // we must not return a signature that will never land on the ER.
  if (txSchedulingSgn.meta.err != null) {
    throw new Error(
      "Transaction failed; commitment was not scheduled (ScheduledCommitSent logs from earlier instructions are not reliable when the transaction errors)",
    );
  }

  const scheduledCommitSgn = parseScheduleCommitsLogsMessage(
    txSchedulingSgn.meta.logMessages ?? [],
  );
  if (scheduledCommitSgn == null) {
    const addresses = (txSchedulingSgn.meta.logMessages ?? []).flatMap((line) => {
      const i = line.indexOf("ScheduledCommitSent outbox: ");
      return i < 0 ? [] : [line.slice(i + "ScheduledCommitSent outbox: ".length).trim()];
    });
    if (addresses.length === 1) {
      const receipt = await waitForIntent(addresses[0], ephemeralConnection);
      if (receipt.signatures.length === 0) throw new Error("Settlement receipt contains no base-chain signature");
      return receipt.signatures[0];
    }
    if (addresses.length > 1) throw new Error("Scheduling transaction created multiple intents; use waitForSettlement");
    throw new Error("ScheduledCommitSent signature not found");
  }

  const latestBlockhash = await ephemeralConnection.getLatestBlockhash();
  await ephemeralConnection.confirmTransaction({
    signature: scheduledCommitSgn,
    ...latestBlockhash,
  });

  const txCommitInfo = await ephemeralConnection.getTransaction(
    scheduledCommitSgn,
    { maxSupportedTransactionVersion: 0 },
  );
  if (txCommitInfo?.meta == null) {
    throw new Error("Transaction not found or meta is null");
  }
  if (txCommitInfo.meta.err != null) {
    throw new Error(
      "Scheduled commit transaction failed; commitment signature is not available",
    );
  }

  const commitSignature = parseCommitsLogsMessage(
    txCommitInfo.meta.logMessages ?? [],
  );
  if (commitSignature == null) {
    throw new Error("Unable to find Commitment signature");
  }

  return commitSignature;
}

export type SettlementReceipt = {
  intentAddress: string | null;
  result: { errorMessage: string | null; logs: string[] };
  signatures: string[];
};

type WaitOptions = { timeoutMs?: number; pollIntervalMs?: number };
const OUTBOX_PREFIX = "ScheduledCommitSent outbox: ";

/** Wait for every settlement scheduled by a successful ER transaction. */
export async function waitForSettlement(
  schedulingSignature: string,
  connection: Connection,
  options: WaitOptions = {},
): Promise<SettlementReceipt[]> {
  const tx = await connection.getTransaction(schedulingSignature, { maxSupportedTransactionVersion: 0 });
  if (!tx?.meta) throw new Error("Scheduling transaction not found");
  if (tx.meta.err != null) throw new Error("Scheduling transaction failed; no settlement receipt exists");
  const logs = tx.meta.logMessages ?? [];
  const addresses = logs.flatMap((line) => {
    const i = line.indexOf(OUTBOX_PREFIX);
    return i < 0 ? [] : [line.slice(i + OUTBOX_PREFIX.length).trim()];
  });
  const legacy = parseScheduleCommitsLogsMessage(logs);
  if (addresses.length === 0 && legacy == null) throw new Error("No settlement receipt found in scheduling logs");
  if (addresses.length > 0) return Promise.all(addresses.map((address) => waitForIntent(address, connection, options)));
  await connection.confirmTransaction(legacy!);
  return [await readLegacyReceipt(legacy!, connection)];
}

/** Wait for a settlement completion recorded in an outbox address's history. */
export async function waitForIntent(
  intentAddress: string,
  connection: Connection,
  options: WaitOptions = {},
): Promise<SettlementReceipt> {
  const timeoutMs = options.timeoutMs ?? 120_000;
  const pollIntervalMs = options.pollIntervalMs ?? 1_000;
  const deadline = Date.now() + timeoutMs;
  while (Date.now() <= deadline) {
    const signatures = await connection.getSignaturesForAddress(new PublicKey(intentAddress), { limit: 1000 });
    for (const item of signatures) {
      const tx = await connection.getTransaction(item.signature, { maxSupportedTransactionVersion: 0 });
      const logs = tx?.meta?.logMessages ?? [];
      if (logs.some((line) => line.includes("ScheduledCommitSent id:"))) {
        return receiptFromLogs(intentAddress, logs);
      }
    }
    await new Promise((resolve) => setTimeout(resolve, pollIntervalMs));
  }
  throw new Error(`Timed out waiting for settlement intent ${intentAddress}; transaction history may be unavailable`);
}

async function readLegacyReceipt(signature: string, connection: Connection): Promise<SettlementReceipt> {
  const tx = await connection.getTransaction(signature, { maxSupportedTransactionVersion: 0 });
  if (!tx?.meta) throw new Error("Legacy settlement notification not found");
  if (tx.meta.err != null) throw new Error("Legacy settlement notification failed");
  return receiptFromLogs(null, tx.meta.logMessages ?? []);
}

export function receiptFromLogs(intentAddress: string | null, logs: string[]): SettlementReceipt {
  const signatures = logs.flatMap((line) => {
    const match = line.match(/ScheduledCommitSent signature\[\d+\]: (.+)$/);
    return match ? [match[1].trim()] : [];
  });
  const errorLine = logs.find((line) => line.includes("ScheduledCommitSent error message:"));
  return { intentAddress, result: { errorMessage: errorLine?.split("ScheduledCommitSent error message:")[1]?.trim() ?? null, logs }, signatures };
}

export function parseScheduleCommitsLogsMessage(logMessages: string[]): string | null {
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
