import bs58 from "bs58";

const OUTBOX_PROGRAM_ID = "outboxintent1111111111111111111111111111111";
const OUTBOX_COMPLETION_TAG = 2;

type AddressLike = string | { toString: () => string };

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

export function logsFromProgram(
  logs: readonly string[],
  programId: string,
): string[] {
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

export function parseOutboxAddresses(logMessages: readonly string[]): string[] {
  const prefix = "Outbox for scheduled intent: ";
  return logMessages.flatMap((line) => {
    const index = line.indexOf(prefix);
    return index < 0 ? [] : [line.slice(index + prefix.length).trim()];
  });
}

export function signatureFromReceipt(
  value: unknown,
  outboxAddress: string,
): string | null {
  if (value == null || typeof value !== "object") return null;
  const tx = value as TransactionWithLogs;
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
        addressToString(instruction.programId) !== OUTBOX_PROGRAM_ID ||
        addressToString(instruction.accounts?.[1]) !== outboxAddress ||
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
      const signature = line.match(
        /Settlement receipt signature\[\d+\]: (.+)$/,
      );
      if (signature != null) return signature[1].trim();
    }
  }
  return null;
}

function addressToString(address: AddressLike | undefined): string | null {
  if (address == null) return null;
  return typeof address === "string" ? address : address.toString();
}
