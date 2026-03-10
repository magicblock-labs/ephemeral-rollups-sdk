import {
  AccountRole,
  Address,
  Instruction,
  getAddressEncoder,
  getProgramDerivedAddress,
} from "@solana/kit";
import { SYSTEM_PROGRAM_ADDRESS } from "@solana-program/system";

import {
  DELEGATION_PROGRAM_ID,
  EPHEMERAL_SPL_TOKEN_PROGRAM_ID,
  MAGIC_CONTEXT_ID,
  MAGIC_PROGRAM_ID,
} from "../../constants";
import {
  delegateBufferPdaFromDelegatedAccountAndOwnerProgram,
  delegationMetadataPdaFromDelegatedAccount,
  delegationRecordPdaFromDelegatedAccount,
} from "../../pda";

const INITIALIZE_TRANSFER_QUEUE_DISCRIMINATOR = 12;
const ENSURE_TRANSFER_QUEUE_CRANK_DISCRIMINATOR = 17;
const DELEGATE_TRANSFER_QUEUE_DISCRIMINATOR = 18;

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
 * @param payer - The payer account
 * @param queue - The transfer queue PDA
 * @param mint - The mint account
 * @param sizeBytes - Optional queue size in bytes. Omit to use the program default.
 * @returns The initialize transfer queue instruction
 */
export function initTransferQueueIx(
  payer: Address,
  queue: Address,
  mint: Address,
  sizeBytes?: number,
): Instruction {
  return {
    accounts: [
      { address: payer, role: AccountRole.WRITABLE_SIGNER },
      { address: queue, role: AccountRole.WRITABLE },
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
 * @param payer - The payer account
 * @param queue - The transfer queue PDA
 * @param taskContext - The Magic task context account
 * @param magicProgram - The Magic program account
 * @returns The ensure transfer queue crank instruction
 */
export function ensureTransferQueueCrankIx(
  payer: Address,
  queue: Address,
  taskContext: Address = MAGIC_CONTEXT_ID,
  magicProgram: Address = MAGIC_PROGRAM_ID,
): Instruction {
  return {
    accounts: [
      { address: payer, role: AccountRole.WRITABLE_SIGNER },
      { address: queue, role: AccountRole.WRITABLE },
      { address: taskContext, role: AccountRole.WRITABLE },
      { address: magicProgram, role: AccountRole.READONLY },
    ],
    data: new Uint8Array([ENSURE_TRANSFER_QUEUE_CRANK_DISCRIMINATOR]),
    programAddress: EPHEMERAL_SPL_TOKEN_PROGRAM_ID,
  };
}

/**
 * Delegate the per-mint transfer queue PDA.
 * @param payer - The payer account
 * @param queue - The transfer queue PDA
 * @param mint - The mint account
 * @param validator - Optional validator address override
 * @returns The delegate transfer queue instruction
 */
export async function delegateTransferQueueIx(
  queue: Address,
  payer: Address,
  mint: Address,
  validator?: Address,
): Promise<Instruction> {
  const addressEncoder = getAddressEncoder();
  const validatorBytes = validator === undefined
    ? []
    : Array.from(addressEncoder.encode(validator));

  return {
    accounts: [
      { address: payer, role: AccountRole.WRITABLE_SIGNER },
      { address: queue, role: AccountRole.WRITABLE },
      { address: mint, role: AccountRole.READONLY },
      { address: EPHEMERAL_SPL_TOKEN_PROGRAM_ID, role: AccountRole.READONLY },
      {
        address: await delegateBufferPdaFromDelegatedAccountAndOwnerProgram(
          queue,
          EPHEMERAL_SPL_TOKEN_PROGRAM_ID,
        ),
        role: AccountRole.WRITABLE,
      },
      {
        address: await delegationRecordPdaFromDelegatedAccount(queue),
        role: AccountRole.WRITABLE,
      },
      {
        address: await delegationMetadataPdaFromDelegatedAccount(queue),
        role: AccountRole.WRITABLE,
      },
      { address: DELEGATION_PROGRAM_ID, role: AccountRole.READONLY },
      { address: SYSTEM_PROGRAM_ADDRESS, role: AccountRole.READONLY },
    ],
    data: new Uint8Array([
      DELEGATE_TRANSFER_QUEUE_DISCRIMINATOR,
      ...validatorBytes,
    ]),
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
