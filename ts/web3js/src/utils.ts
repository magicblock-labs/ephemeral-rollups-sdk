// eslint-disable-next-line @typescript-eslint/consistent-type-imports
import { Connection, PublicKey } from "@solana/web3.js";
import bs58 from "bs58";
import { MAGIC_PROGRAM_ID } from "./constants";

const OUTBOX_PROGRAM_ID = "outboxintent1111111111111111111111111111111";
const OUTBOX_COMPLETION_TAG = 2;
const SETTLEMENT_TIMEOUT_MS = 120_000;
const POLL_INTERVAL_MS = 1_000;

type AddressLike = string | { toBase58: () => string };

interface TransactionWithLogs {
  transaction?: {
    message?: {
      instructions?: Array<{
        programId?: AddressLike;
        accounts?: AddressLike[];
        data?: string;
      }>;
    };
  };
  meta?: {
    err: unknown | null;
    logMessages?: string[] | null;
  } | null;
}

/**
 * Get the signature of the accounts commitment transaction to the base layer.
 *
 * Validator v1 logs commit signatures directly; v2 publishes them in an outbox
 * settlement receipt. This method retains its single-signature return type.
 */
export async function GetCommitmentSignature(
  transactionSignature: string,
  ephemeralConnection: Connection,
): Promise<string> {
  const schedulingTx = await getTransactionWithLogs(
    ephemeralConnection,
    transactionSignature,
  );
  if (schedulingTx?.meta == null) {
    throw new Error("Transaction not found or meta is null");
  }
  if (schedulingTx.meta.err != null) {
    throw new Error("Transaction failed; commitment was not scheduled");
  }

  const magicLogs = logsFromProgram(
    schedulingTx.meta.logMessages ?? [],
    MAGIC_PROGRAM_ID.toBase58(),
  );

  // Validator v1 embeds the scheduled commit signature in the scheduling logs.
  const scheduledCommitSignature = parseScheduleCommitsLogsMessage(magicLogs);
  if (scheduledCommitSignature != null) {
    const latestBlockhash = await ephemeralConnection.getLatestBlockhash();
    await ephemeralConnection.confirmTransaction({
      signature: scheduledCommitSignature,
      ...latestBlockhash,
    });

    const commitTx = await getTransactionWithLogs(
      ephemeralConnection,
      scheduledCommitSignature,
    );
    if (commitTx?.meta == null) {
      throw new Error("Transaction not found or meta is null");
    }
    if (commitTx.meta.err != null) {
      throw new Error(
        "Scheduled commit transaction failed; commitment signature is not available",
      );
    }

    const commitmentSignature = parseCommitsLogsMessage(
      logsFromProgram(
        commitTx.meta.logMessages ?? [],
        MAGIC_PROGRAM_ID.toBase58(),
      ),
    );
    if (commitmentSignature == null) {
      throw new Error("Unable to find Commitment signature");
    }
    return commitmentSignature;
  }

  // Validator v2 creates an outbox; its later receipt contains the base signature.
  const outboxAddresses = parseOutboxAddresses(magicLogs);
  if (outboxAddresses.length > 1) {
    throw new Error(
      "Scheduling transaction created multiple intents; this method returns one signature",
    );
  }
  if (outboxAddresses.length === 0) {
    throw new Error(
      "Scheduled commit signature or settlement outbox not found",
    );
  }

  return waitForOutboxSignature(
    outboxAddresses[0],
    ephemeralConnection,
  );
}

/**
 * web3.js 1.x rejects v1 transaction responses before returning their logs.
 * Use its configured RPC transport and read only the response metadata needed here.
 */
async function getTransactionWithLogs(
  connection: Connection,
  signature: string,
): Promise<TransactionWithLogs | null> {
  const commitment = connection._commitment;
  if (
    commitment != null &&
    commitment !== "confirmed" &&
    commitment !== "finalized"
  ) {
    throw new Error(
      `getTransaction requires confirmed or finalized commitment, received ${commitment}`,
    );
  }

  const config = {
    encoding: "jsonParsed",
    // Version 1 includes legacy and version 0 transactions as well.
    maxSupportedTransactionVersion: 1,
    ...(commitment == null ? {} : { commitment }),
  };
  const response = (await connection._rpcRequest("getTransaction", [
    signature,
    config,
  ])) as {
    result?: TransactionWithLogs | null;
    error?: { message?: string };
  };

  if (response.error != null) {
    throw new Error(response.error.message ?? "Failed to get transaction");
  }
  return response.result ?? null;
}

async function waitForOutboxSignature(
  outboxAddress: string,
  connection: Connection,
): Promise<string> {
  const outbox = new PublicKey(outboxAddress);
  const deadline = Date.now() + SETTLEMENT_TIMEOUT_MS;
  const inspected = new Set<string>();

  while (Date.now() <= deadline) {
    const signatures = await connection.getSignaturesForAddress(outbox, {
      limit: 1_000,
    });

    for (const item of signatures) {
      if (inspected.has(item.signature)) continue;

      let tx: TransactionWithLogs | null;
      try {
        tx = await getTransactionWithLogs(connection, item.signature);
      } catch {
        continue;
      }
      if (tx?.meta == null) continue;
      inspected.add(item.signature);

      if (tx.meta.err !== null) continue;
      const signature = signatureFromReceipt(tx, outboxAddress);
      if (signature != null) return signature;
    }

    const remaining = deadline - Date.now();
    if (remaining > 0) {
      await new Promise<void>((resolve) =>
        setTimeout(resolve, Math.min(POLL_INTERVAL_MS, remaining)),
      );
    }
  }

  throw new Error(`Timed out waiting for settlement outbox ${outboxAddress}`);
}

function logsFromProgram(logs: readonly string[], programId: string): string[] {
  const stack: string[] = [];
  const programLogs: string[] = [];

  for (const line of logs) {
    const invoke = line.match(/^Program (\S+) invoke \[\d+\]$/);
    if (invoke != null) {
      stack.push(invoke[1]);
      continue;
    }
    const end = line.match(/^Program (\S+) (?:success|failed:.*)$/);
    if (end != null) {
      const index = stack.lastIndexOf(end[1]);
      if (index >= 0) stack.length = index;
      continue;
    }
    if (
      stack[stack.length - 1] === programId &&
      (line.startsWith("Program log: ") || !line.startsWith("Program "))
    ) {
      programLogs.push(line);
    }
  }
  return programLogs;
}

function parseOutboxAddresses(logMessages: readonly string[]): string[] {
  const prefix = "Outbox for scheduled intent: ";
  return logMessages.flatMap((line) => {
    const index = line.indexOf(prefix);
    return index < 0 ? [] : [line.slice(index + prefix.length).trim()];
  });
}

function signatureFromReceipt(
  tx: TransactionWithLogs,
  outboxAddress: string,
): string | null {
  const meta = tx.meta;
  const instructions = tx.transaction?.message?.instructions ?? [];
  if (meta == null || meta.err !== null) return null;

  const logs = logsFromProgram(meta.logMessages ?? [], OUTBOX_PROGRAM_ID);
  const receiptStart =
    /^(?:Program log: )?Settlement receipt intent id: (\d+),/;
  let currentReceipt: string[] = [];
  const receipts: string[][] = [];

  for (const line of logs) {
    if (receiptStart.test(line)) {
      if (currentReceipt.length > 0) receipts.push(currentReceipt);
      currentReceipt = [line];
    } else if (currentReceipt.length > 0) {
      currentReceipt.push(line);
    }
  }
  if (currentReceipt.length > 0) receipts.push(currentReceipt);

  for (const receipt of receipts) {
    const idMatch = receipt[0]?.match(receiptStart);
    if (idMatch == null) continue;
    const intentId = BigInt(idMatch[1]);
    const isOutboxCompletion = instructions.some((instruction) => {
      if (
        addressToBase58(instruction.programId) !== OUTBOX_PROGRAM_ID ||
        addressToBase58(instruction.accounts?.[1]) !== outboxAddress ||
        instruction.data == null
      ) {
        return false;
      }

      let data: Uint8Array;
      try {
        data = bs58.decode(instruction.data);
      } catch {
        return false;
      }
      if (
        data.length < 12 ||
        data[0] !== OUTBOX_COMPLETION_TAG ||
        data[1] !== 0 ||
        data[2] !== 0 ||
        data[3] !== 0
      ) {
        return false;
      }

      let instructionId = 0n;
      for (let byte = 7; byte >= 0; byte--) {
        instructionId = (instructionId << 8n) | BigInt(data[4 + byte]);
      }
      return instructionId === intentId;
    });

    if (!isOutboxCompletion) continue;
    for (const line of receipt) {
      const match = line.match(/Settlement receipt signature\[\d+\]: (.+)$/);
      if (match != null) return match[1].trim();
    }
  }
  return null;
}

function addressToBase58(address: AddressLike | undefined): string | null {
  if (typeof address === "string") return address;
  return address?.toBase58() ?? null;
}

function parseScheduleCommitsLogsMessage(
  logMessages: readonly string[],
): string | null {
  const signaturePrefixes = [
    // Preserve the v1 validator's original log label.
    "ScheduledCommitSent signature[0]: ",
    "ScheduledCommitSent signature: ",
  ];
  for (const message of logMessages) {
    for (const signaturePrefix of signaturePrefixes) {
      const index = message.indexOf(signaturePrefix);
      if (index >= 0) {
        return message.slice(index + signaturePrefix.length).trim();
      }
    }
  }
  return null;
}

function parseCommitsLogsMessage(
  logMessages: readonly string[],
): string | null {
  const signaturePrefix = "ScheduledCommitSent signature[0]: ";
  for (const message of logMessages) {
    const index = message.indexOf(signaturePrefix);
    if (index >= 0) return message.slice(index + signaturePrefix.length).trim();
  }
  return null;
}
