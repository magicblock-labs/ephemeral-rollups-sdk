// eslint-disable-next-line @typescript-eslint/consistent-type-imports
import { Connection } from "@solana/web3.js";

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

function parseScheduleCommitsLogsMessage(logMessages: string[]): string | null {
  for (const message of logMessages) {
    const signaturePrefix = "ScheduledCommitSent signature: ";
    if (message.includes(signaturePrefix)) {
      return message.split(signaturePrefix)[1];
    }
  }
  return null;
}

function parseCommitsLogsMessage(logMessages: string[]): string | null {
  for (const message of logMessages) {
    const signaturePrefix = "ScheduledCommitSent signature[0]: ";
    if (message.includes(signaturePrefix)) {
      return message.split(signaturePrefix)[1];
    }
  }
  return null;
}
