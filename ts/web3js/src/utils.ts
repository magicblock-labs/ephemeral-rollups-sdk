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
    const addresses = parseOutboxAddresses(
      txSchedulingSgn.meta.logMessages ?? [],
    );
    if (addresses.length === 1) {
      const receipt = await waitForIntent(addresses[0], ephemeralConnection);
      if (receipt.signatures.length === 0)
        throw new Error("Settlement receipt contains no base-chain signature");
      return receipt.signatures[0];
    }
    if (addresses.length > 1)
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

export interface SettlementReceipt {
  intentAddress: string | null;
  result: { errorMessage: string | null; logs: string[] };
  signatures: string[];
}

interface WaitOptions {
  timeoutMs?: number;
  pollIntervalMs?: number;
  signal?: AbortSignal;
}
const OUTBOX_PREFIX = "ScheduledCommitSent outbox: ";
const OUTBOX_PROGRAM_ID = "outboxintent1111111111111111111111111111111";

async function withDeadline<T>(
  promise: Promise<T>,
  deadline: number,
  signal?: AbortSignal,
): Promise<T> {
  const remaining = deadline - Date.now();
  if (remaining <= 0)
    return Promise.reject(
      new Error("Timed out waiting for settlement receipt"),
    );
  return new Promise<T>((resolve, reject) => {
    const cleanup = () => {
      clearTimeout(timer);
      signal?.removeEventListener("abort", onAbort);
    };
    const timer = setTimeout(() => {
      cleanup();
      reject(new Error("Timed out waiting for settlement receipt"));
    }, remaining);
    const onAbort = () => {
      cleanup();
      reject(new Error("Settlement receipt wait cancelled"));
    };
    signal?.addEventListener("abort", onAbort, { once: true });
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
  options: WaitOptions = {},
): Promise<SettlementReceipt[]> {
  const tx = await connection.getTransaction(schedulingSignature, {
    maxSupportedTransactionVersion: 0,
  });
  if (!tx?.meta) throw new Error("Scheduling transaction not found");
  if (tx.meta.err != null)
    throw new Error(
      "Scheduling transaction failed; no settlement receipt exists",
    );
  const logs = tx.meta.logMessages ?? [];
  const addresses = parseOutboxAddresses(logs);
  if (addresses.length > 0) {
    const controller = new AbortController();
    const tasks = addresses.map(async (address) =>
      waitForIntent(address, connection, {
        ...options,
        signal: controller.signal,
      }),
    );
    try {
      return await Promise.all(tasks);
    } catch (error) {
      controller.abort();
      await Promise.allSettled(tasks);
      throw error;
    }
  }
  const legacy = parseScheduleCommitsLogsMessage(logs);
  if (legacy == null)
    throw new Error("No settlement receipt found in scheduling logs");
  await connection.confirmTransaction(legacy);
  return [await readLegacyReceipt(legacy, connection)];
}

/** Wait for a settlement completion recorded in an outbox address's history. */
export async function waitForIntent(
  intentAddress: string,
  connection: Connection,
  options: WaitOptions = {},
): Promise<SettlementReceipt> {
  let address: PublicKey;
  try {
    address = new PublicKey(intentAddress);
  } catch {
    throw new Error(`Invalid settlement intent address: ${intentAddress}`);
  }
  const timeoutMs = options.timeoutMs ?? 120_000;
  const pollIntervalMs = options.pollIntervalMs ?? 1_000;
  const deadline = Date.now() + timeoutMs;
  const inspected = new Set<string>();
  const rpcRequest = (
    connection as unknown as {
      _rpcRequest: (
        method: string,
        args: unknown[],
      ) => Promise<{ result?: { identity?: string } }>;
    }
  )._rpcRequest.bind(connection);
  const identityResult = await withDeadline(
    rpcRequest("getIdentity", []),
    deadline,
    options.signal,
  );
  if (
    identityResult.result?.identity == null ||
    identityResult.result.identity === ""
  )
    throw new Error(
      "Could not determine validator identity for receipt verification",
    );
  const validatorIdentity = new PublicKey(identityResult.result.identity);
  while (Date.now() <= deadline) {
    if (options.signal?.aborted === true)
      throw new Error("Settlement receipt wait cancelled");
    const signatures = await withDeadline(
      connection.getSignaturesForAddress(address, { limit: 1000 }),
      deadline,
      options.signal,
    );
    for (const item of signatures) {
      if (inspected.has(item.signature)) continue;
      let tx;
      try {
        tx = await withDeadline(
          connection.getTransaction(item.signature, {
            maxSupportedTransactionVersion: 0,
          }),
          deadline,
          options.signal,
        );
      } catch (error) {
        if (Date.now() >= deadline) throw error;
        continue;
      }
      if (tx?.meta != null) inspected.add(item.signature);
      const logs = tx?.meta?.logMessages ?? [];
      if (
        tx?.meta != null &&
        tx.meta.err == null &&
        hasAuthorizedCompletion(tx, validatorIdentity, address) &&
        logs.some((line) => line.includes("ScheduledCommitSent id:"))
      ) {
        return receiptFromLogs(intentAddress, logs);
      }
    }
    await withDeadline(
      new Promise<void>((resolve) =>
        setTimeout(
          resolve,
          Math.min(pollIntervalMs, Math.max(0, deadline - Date.now())),
        ),
      ),
      deadline,
      options.signal,
    );
  }
  throw new Error(
    `Timed out waiting for settlement intent ${intentAddress}; transaction history may be unavailable`,
  );
}

export function hasAuthorizedCompletion(
  tx: unknown,
  validator: PublicKey,
  outbox: PublicKey,
): boolean {
  const confirmedTx = tx as NonNullable<
    Awaited<ReturnType<Connection["getTransaction"]>>
  >;
  const message = confirmedTx.transaction.message as unknown as {
    staticAccountKeys?: PublicKey[];
    accountKeys?: Array<PublicKey | { pubkey: PublicKey; signer?: boolean }>;
    header?: { numRequiredSignatures?: number };
    compiledInstructions?: Array<{
      programIdIndex: number;
      accountKeyIndexes: number[];
    }>;
    instructions?: Array<{
      programId?: PublicKey;
      programIdIndex?: number;
      accounts?: Array<PublicKey | number>;
    }>;
  };
  const keys =
    message.staticAccountKeys ??
    message.accountKeys?.map((key) =>
      key instanceof PublicKey ? key : key.pubkey,
    ) ??
    [];
  const signerCount = message.header?.numRequiredSignatures ?? 0;
  if (!keys.slice(0, signerCount).some((key) => key.equals(validator)))
    return false;
  const outboxIndex = keys.findIndex((key) => key.equals(outbox));
  const instructions =
    message.compiledInstructions ?? message.instructions ?? [];
  return (
    outboxIndex >= 0 &&
    instructions.some((ix) => {
      const program =
        ("programId" in ix ? ix.programId : undefined) ??
        keys[ix.programIdIndex ?? -1];
      const accounts =
        "accountKeyIndexes" in ix
          ? ix.accountKeyIndexes.map((index) => keys[index])
          : (ix.accounts ?? []).map((account) =>
              typeof account === "number" ? keys[account] : account,
            );
      return (
        program?.toBase58() === OUTBOX_PROGRAM_ID &&
        accounts.some((key) => key?.equals(outbox))
      );
    })
  );
}

async function readLegacyReceipt(
  signature: string,
  connection: Connection,
): Promise<SettlementReceipt> {
  const tx = await connection.getTransaction(signature, {
    maxSupportedTransactionVersion: 0,
  });
  if (!tx?.meta) throw new Error("Legacy settlement notification not found");
  if (tx.meta.err != null)
    throw new Error("Legacy settlement notification failed");
  return receiptFromLogs(null, tx.meta.logMessages ?? []);
}

export function receiptFromLogs(
  intentAddress: string | null,
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
