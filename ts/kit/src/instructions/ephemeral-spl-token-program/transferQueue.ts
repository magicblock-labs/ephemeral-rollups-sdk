import {
  AccountRole,
  Address,
  Instruction,
  getAddressEncoder,
  getProgramDerivedAddress,
} from "@solana/kit";
import { SYSTEM_PROGRAM_ADDRESS } from "@solana-program/system";

import {
  EPHEMERAL_SPL_TOKEN_PROGRAM_ID,
  MAGIC_CONTEXT_ID,
  MAGIC_PROGRAM_ID,
} from "../../constants";

const INITIALIZE_TRANSFER_QUEUE_DISCRIMINATOR = 12;
const ENSURE_TRANSFER_QUEUE_CRANK_DISCRIMINATOR = 17;

/**
 * Derive the transfer queue PDA for a mint.
 * @param mint - The mint account address
 * @returns The transfer queue PDA and bump
 */
export async function deriveTransferQueue(
  mint: Address,
): Promise<[Address, number]> {
  const addressEncoder = getAddressEncoder();
  const [queue, bump] = await getProgramDerivedAddress({
    programAddress: EPHEMERAL_SPL_TOKEN_PROGRAM_ID,
    seeds: [new Uint8Array(Buffer.from("queue")), addressEncoder.encode(mint)],
  });
  return [queue, bump];
}

/**
 * Initialize the per-mint transfer queue.
 * @param queue - The transfer queue PDA
 * @param payer - The payer account
 * @param mint - The mint account
 * @param sizeBytes - Optional queue size in bytes. Omit to use the program default.
 * @returns The initialize transfer queue instruction
 */
export function initTransferQueueIx(
  queue: Address,
  payer: Address,
  mint: Address,
  sizeBytes?: number,
): Instruction {
  return {
    accounts: [
      { address: queue, role: AccountRole.WRITABLE },
      { address: payer, role: AccountRole.WRITABLE_SIGNER },
      { address: mint, role: AccountRole.READONLY },
      { address: SYSTEM_PROGRAM_ADDRESS, role: AccountRole.READONLY },
    ],
    data:
      sizeBytes === undefined
        ? new Uint8Array([INITIALIZE_TRANSFER_QUEUE_DISCRIMINATOR])
        : new Uint8Array([INITIALIZE_TRANSFER_QUEUE_DISCRIMINATOR, ...u32le(sizeBytes)]),
    programAddress: EPHEMERAL_SPL_TOKEN_PROGRAM_ID,
  };
}

/**
 * Ensure the recurring transfer queue crank is scheduled.
 * @param queue - The transfer queue PDA
 * @param payer - The payer account
 * @param taskContext - The Magic task context account
 * @param magicProgram - The Magic program account
 * @returns The ensure transfer queue crank instruction
 */
export function ensureTransferQueueCrankIx(
  queue: Address,
  payer: Address,
  taskContext: Address = MAGIC_CONTEXT_ID,
  magicProgram: Address = MAGIC_PROGRAM_ID,
): Instruction {
  return {
    accounts: [
      { address: queue, role: AccountRole.WRITABLE },
      { address: payer, role: AccountRole.WRITABLE_SIGNER },
      { address: taskContext, role: AccountRole.WRITABLE },
      { address: magicProgram, role: AccountRole.READONLY },
    ],
    data: new Uint8Array([ENSURE_TRANSFER_QUEUE_CRANK_DISCRIMINATOR]),
    programAddress: EPHEMERAL_SPL_TOKEN_PROGRAM_ID,
  };
}

function u32le(n: number): number[] {
  if (!Number.isInteger(n) || n < 0 || n > 0xffff_ffff) {
    throw new Error("sizeBytes out of range for u32");
  }

  return [
    n & 0xff,
    (n >>> 8) & 0xff,
    (n >>> 16) & 0xff,
    (n >>> 24) & 0xff,
  ];
}
