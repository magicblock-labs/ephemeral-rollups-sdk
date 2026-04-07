import {
  AccountRole,
  Address,
  Instruction,
  address,
  getAddressEncoder,
  getProgramDerivedAddress,
} from "@solana/kit";
import { SYSTEM_PROGRAM_ADDRESS } from "@solana-program/system";

import {
  DELEGATION_PROGRAM_ID,
  EPHEMERAL_SPL_TOKEN_PROGRAM_ID,
  MAGIC_CONTEXT_ID,
  MAGIC_PROGRAM_ID,
  PERMISSION_PROGRAM_ID,
} from "../../constants";
import {
  delegateBufferPdaFromDelegatedAccountAndOwnerProgram,
  delegationMetadataPdaFromDelegatedAccount,
  delegationRecordPdaFromDelegatedAccount,
  permissionPdaFromAccount,
} from "../../pda";

const INITIALIZE_TRANSFER_QUEUE_DISCRIMINATOR = 12;
const DEPOSIT_AND_QUEUE_TRANSFER_DISCRIMINATOR = 16;
const ENSURE_TRANSFER_QUEUE_CRANK_DISCRIMINATOR = 17;
const DELEGATE_TRANSFER_QUEUE_DISCRIMINATOR = 19;
const ALLOCATE_TRANSFER_QUEUE_DISCRIMINATOR = 27;
const PROCESS_PENDING_TRANSFER_QUEUE_REFILL_DISCRIMINATOR = 28;
const QUEUE_SEED = new TextEncoder().encode("queue");
const QUEUE_REFILL_STATE_SEED = new TextEncoder().encode("queue-refill");
const RENT_PDA_SEED = new TextEncoder().encode("rent");
const LAMPORTS_PDA_SEED = new TextEncoder().encode("lamports");
const TOKEN_PROGRAM_ADDRESS = address(
  "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA",
);

/**
 * Derive the transfer queue PDA for a mint/validator pair.
 * @param mint - The mint account address
 * @param validator - The validator account address
 * @returns The transfer queue PDA and bump
 */
export async function deriveTransferQueue(
  mint: Address,
  validator: Address,
): Promise<[Address, number]> {
  const addressEncoder = getAddressEncoder();
  const [queue, bump] = await getProgramDerivedAddress({
    programAddress: EPHEMERAL_SPL_TOKEN_PROGRAM_ID,
    seeds: [
      QUEUE_SEED,
      addressEncoder.encode(mint),
      addressEncoder.encode(validator),
    ],
  });
  return [queue, bump];
}

/**
 * Initialize the per-validator transfer queue for a mint.
 * @param payer - The payer account
 * @param queue - The transfer queue PDA
 * @param mint - The mint account
 * @param validator - The validator account
 * @param requestedItems - Optional queue item count. Omit to use the program default.
 * @returns The initialize transfer queue instruction
 */
export async function initTransferQueueIx(
  payer: Address,
  queue: Address,
  mint: Address,
  validator: Address,
  requestedItems?: number,
): Promise<Instruction> {
  return {
    accounts: [
      { address: payer, role: AccountRole.WRITABLE_SIGNER },
      { address: queue, role: AccountRole.WRITABLE },
      {
        address: await permissionPdaFromAccount(queue),
        role: AccountRole.WRITABLE,
      },
      { address: mint, role: AccountRole.READONLY },
      { address: validator, role: AccountRole.READONLY },
      { address: SYSTEM_PROGRAM_ADDRESS, role: AccountRole.READONLY },
      { address: PERMISSION_PROGRAM_ID, role: AccountRole.READONLY },
    ],
    data:
      requestedItems === undefined
        ? new Uint8Array([INITIALIZE_TRANSFER_QUEUE_DISCRIMINATOR])
        : new Uint8Array([
            INITIALIZE_TRANSFER_QUEUE_DISCRIMINATOR,
            ...u32le(requestedItems),
          ]),
    programAddress: EPHEMERAL_SPL_TOKEN_PROGRAM_ID,
  };
}

/**
 * Allocate additional space for a prefunded transfer queue.
 * @param queue - The transfer queue PDA
 * @returns The allocate transfer queue instruction
 */
export function allocateTransferQueueIx(queue: Address): Instruction {
  return {
    accounts: [
      { address: queue, role: AccountRole.WRITABLE },
      { address: SYSTEM_PROGRAM_ADDRESS, role: AccountRole.READONLY },
    ],
    data: new Uint8Array([ALLOCATE_TRANSFER_QUEUE_DISCRIMINATOR]),
    programAddress: EPHEMERAL_SPL_TOKEN_PROGRAM_ID,
  };
}

/**
 * Deposit SPL tokens into the vault and queue one or more delayed transfers.
 * @param queue - The transfer queue PDA
 * @param vault - The mint vault PDA
 * @param mint - The mint account
 * @param source - The sender token account
 * @param vaultAta - The vault token account
 * @param destination - The queued destination owner
 * @param owner - The sender authority
 * @param amount - The total amount to queue
 * @param minDelayMs - The minimum delay in milliseconds
 * @param maxDelayMs - The maximum delay in milliseconds
 * @param split - The number of queue entries to create
 * @param reimbursementTokenInfo - Reimbursement token account used by the queue-full fallback path
 * @param clientRefId - Optional client-provided reference ID attached to each queued split
 * @returns The deposit-and-queue-transfer instruction
 */
export function depositAndQueueTransferIx(
  queue: Address,
  vault: Address,
  mint: Address,
  source: Address,
  vaultAta: Address,
  destination: Address,
  owner: Address,
  amount: bigint,
  minDelayMs: bigint = 0n,
  maxDelayMs: bigint = minDelayMs,
  split: number = 1,
  reimbursementTokenInfo: Address = source,
  clientRefId?: bigint,
): Instruction {
  if (!Number.isInteger(split) || split <= 0 || split > 0xffff_ffff) {
    throw new Error("split must fit in u32");
  }
  if (
    amount < 0n ||
    minDelayMs < 0n ||
    maxDelayMs < 0n ||
    (clientRefId !== undefined && clientRefId < 0n)
  ) {
    throw new Error("amount, delays, and clientRefId must be non-negative");
  }
  if (maxDelayMs < minDelayMs) {
    throw new Error("maxDelayMs must be greater than or equal to minDelayMs");
  }

  const data = [
    DEPOSIT_AND_QUEUE_TRANSFER_DISCRIMINATOR,
    ...u64le(amount),
    ...u64le(minDelayMs),
    ...u64le(maxDelayMs),
    ...u32le(split),
  ];
  if (clientRefId !== undefined) {
    data.push(...u64le(clientRefId));
  }

  return {
    accounts: [
      { address: queue, role: AccountRole.WRITABLE },
      { address: vault, role: AccountRole.READONLY },
      { address: mint, role: AccountRole.READONLY },
      { address: source, role: AccountRole.WRITABLE },
      { address: vaultAta, role: AccountRole.WRITABLE },
      { address: destination, role: AccountRole.READONLY },
      { address: owner, role: AccountRole.READONLY_SIGNER },
      { address: TOKEN_PROGRAM_ADDRESS, role: AccountRole.READONLY },
      { address: reimbursementTokenInfo, role: AccountRole.WRITABLE },
    ],
    data: new Uint8Array(data),
    programAddress: EPHEMERAL_SPL_TOKEN_PROGRAM_ID,
  };
}

/**
 * Ensure the recurring transfer queue crank is scheduled.
 * @param payer - The payer account
 * @param queue - The transfer queue PDA
 * @param magicFeeVault - The validator magic fee vault PDA from the delegation program
 * @param magicContext - The Magic context account
 * @param magicProgram - The Magic program account
 * @returns The ensure transfer queue crank instruction
 */
export function ensureTransferQueueCrankIx(
  payer: Address,
  queue: Address,
  magicFeeVault: Address,
  magicContext: Address = MAGIC_CONTEXT_ID,
  magicProgram: Address = MAGIC_PROGRAM_ID,
): Instruction {
  return {
    accounts: [
      { address: payer, role: AccountRole.WRITABLE_SIGNER },
      { address: queue, role: AccountRole.WRITABLE },
      { address: magicFeeVault, role: AccountRole.WRITABLE },
      { address: magicContext, role: AccountRole.WRITABLE },
      { address: magicProgram, role: AccountRole.READONLY },
    ],
    data: new Uint8Array([ENSURE_TRANSFER_QUEUE_CRANK_DISCRIMINATOR]),
    programAddress: EPHEMERAL_SPL_TOKEN_PROGRAM_ID,
  };
}

/**
 * Delegate the per-mint transfer queue PDA.
 * @param queue - The transfer queue PDA
 * @param payer - The payer account
 * @param mint - The mint account
 * @returns The delegate transfer queue instruction
 */
export async function delegateTransferQueueIx(
  queue: Address,
  payer: Address,
  mint: Address,
): Promise<Instruction> {
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
    data: new Uint8Array([DELEGATE_TRANSFER_QUEUE_DISCRIMINATOR]),
    programAddress: EPHEMERAL_SPL_TOKEN_PROGRAM_ID,
  };
}

/**
 * Execute a pending transfer-queue refill through the sponsored lamports flow.
 * @param queue - The delegated transfer queue PDA
 * @returns The pending transfer queue refill instruction
 */
export async function processPendingTransferQueueRefillIx(
  queue: Address,
): Promise<Instruction> {
  const addressEncoder = getAddressEncoder();
  const [rentPda] = await getProgramDerivedAddress({
    programAddress: EPHEMERAL_SPL_TOKEN_PROGRAM_ID,
    seeds: [RENT_PDA_SEED],
  });
  const [refillState] = await getProgramDerivedAddress({
    programAddress: EPHEMERAL_SPL_TOKEN_PROGRAM_ID,
    seeds: [QUEUE_REFILL_STATE_SEED, addressEncoder.encode(queue)],
  });
  const queueBytes = addressEncoder.encode(queue);
  const [lamportsPda] = await getProgramDerivedAddress({
    programAddress: EPHEMERAL_SPL_TOKEN_PROGRAM_ID,
    seeds: [
      LAMPORTS_PDA_SEED,
      addressEncoder.encode(rentPda),
      queueBytes,
      queueBytes,
    ],
  });

  return {
    accounts: [
      { address: refillState, role: AccountRole.WRITABLE },
      { address: queue, role: AccountRole.WRITABLE },
      { address: rentPda, role: AccountRole.WRITABLE },
      { address: lamportsPda, role: AccountRole.WRITABLE },
      { address: EPHEMERAL_SPL_TOKEN_PROGRAM_ID, role: AccountRole.READONLY },
      {
        address: await delegateBufferPdaFromDelegatedAccountAndOwnerProgram(
          lamportsPda,
          EPHEMERAL_SPL_TOKEN_PROGRAM_ID,
        ),
        role: AccountRole.WRITABLE,
      },
      {
        address: await delegationRecordPdaFromDelegatedAccount(lamportsPda),
        role: AccountRole.WRITABLE,
      },
      {
        address: await delegationMetadataPdaFromDelegatedAccount(lamportsPda),
        role: AccountRole.WRITABLE,
      },
      { address: DELEGATION_PROGRAM_ID, role: AccountRole.READONLY },
      { address: SYSTEM_PROGRAM_ADDRESS, role: AccountRole.READONLY },
      {
        address: await delegationRecordPdaFromDelegatedAccount(queue),
        role: AccountRole.READONLY,
      },
    ],
    data: new Uint8Array([PROCESS_PENDING_TRANSFER_QUEUE_REFILL_DISCRIMINATOR]),
    programAddress: EPHEMERAL_SPL_TOKEN_PROGRAM_ID,
  };
}

function u32le(n: number): number[] {
  if (!Number.isInteger(n) || n < 0 || n > 0xffff_ffff) {
    throw new Error("value out of range for u32");
  }

  return [n & 0xff, (n >>> 8) & 0xff, (n >>> 16) & 0xff, (n >>> 24) & 0xff];
}

function u64le(n: bigint): number[] {
  if (n < 0n || n > 0xffff_ffff_ffff_ffffn) {
    throw new Error("value out of range for u64");
  }

  const out = Buffer.alloc(8);
  out.writeBigUInt64LE(n);
  return Array.from(out);
}
